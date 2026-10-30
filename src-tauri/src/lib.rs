use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tauri::State;

mod ai;

use ai::{parse_ollama_tags, recommend_model, validate_quotes, TrustedSource};

static COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Default)]
struct Job {
    lines: Vec<String>,
    done: bool,
    codes: Vec<i32>,
}

#[derive(Default)]
struct AppState {
    files: Mutex<HashMap<String, PathBuf>>,
    jobs: Mutex<HashMap<String, Arc<Mutex<Job>>>>,
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn new_id() -> String {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let t = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut h = Sha256::new();
    h.update(format!("{}-{}-{}", n, t, std::process::id()));
    hex(&h.finalize())[..12].to_string()
}

fn hidden(program: &str) -> Command {
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut c = Command::new(program);
    #[cfg(windows)]
    {
        c.creation_flags(0x0800_0000);
    }
    c
}

fn valid_id(s: &str) -> bool {
    let first_ok = s.starts_with(|c: char| c.is_ascii_alphanumeric());
    first_ok
        && s.len() <= 100
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
}

fn env_or(k: &str) -> String {
    std::env::var(k).unwrap_or_default()
}

fn ext_of(p: &Path) -> String {
    p.extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_ascii_lowercase())
        .unwrap_or_default()
}

fn clean_lines(text: &str) -> Vec<String> {
    text.split('\n')
        .filter_map(|l| {
            let seg = l.rsplit('\r').find(|s| !s.trim().is_empty())?;
            let t = seg.trim_end();
            if t.trim().chars().count() <= 1 || t.contains(['█', '▒', '▓', '░']) {
                return None;
            }
            Some(t.to_string())
        })
        .collect()
}

fn find_word(h: &[char], word: &str) -> Option<usize> {
    let w: Vec<char> = word.chars().collect();
    if h.len() < w.len() {
        return None;
    }
    for i in 0..=(h.len() - w.len()) {
        let slice: String = h[i..i + w.len()].iter().collect();
        if slice.eq_ignore_ascii_case(word) {
            let before_ok = i == 0 || h[i - 1] == ' ';
            let after_ok = i + w.len() == h.len() || h[i + w.len()] == ' ';
            if before_ok && after_ok {
                return Some(i);
            }
        }
    }
    None
}

fn first_token(c: &[char]) -> String {
    let s: String = c.iter().collect();
    s.split_whitespace().next().unwrap_or("").to_string()
}

fn parse_table(lines: &[String]) -> Vec<Value> {
    let di = match lines
        .iter()
        .position(|l| l.trim().len() >= 5 && l.trim().chars().all(|c| c == '-'))
    {
        Some(i) if i >= 1 => i,
        _ => return vec![],
    };
    let header: Vec<char> = lines[di - 1].chars().collect();
    let id_i = match find_word(&header, "id") {
        Some(i) => i,
        None => return vec![],
    };
    let ver_i = find_word(&header, "version");
    let mut res = vec![];
    for l in lines.iter().skip(di + 1) {
        if res.len() >= 15 {
            break;
        }
        let c: Vec<char> = l.chars().collect();
        if c.len() <= id_i {
            continue;
        }
        let name: String = c[..id_i].iter().collect::<String>().trim().to_string();
        let (id, ver) = match ver_i {
            Some(v) if v > id_i && c.len() > v => (
                c[id_i..v].iter().collect::<String>().trim().to_string(),
                first_token(&c[v..]),
            ),
            _ => (first_token(&c[id_i..]), String::new()),
        };
        if valid_id(&id) {
            res.push(json!({ "id": id, "nm": name, "ver": ver }));
        }
    }
    res
}

fn winget_version() -> Option<String> {
    hidden("winget")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn elevation() -> &'static str {
    match hidden("whoami").arg("/groups").output() {
        Ok(o) if o.status.success() => {
            let t = String::from_utf8_lossy(&o.stdout);
            if t.contains("S-1-16-12288") {
                "elevated"
            } else if t.contains("S-1-5-32-544") {
                "admin_not_elevated"
            } else {
                "standard"
            }
        }
        _ => "unknown",
    }
}

fn cap(state: &str, reason: &str) -> Value {
    json!({ "state": state, "reason": reason })
}

fn cap_exe() -> Value {
    cap("available", "Das Programm startet mit deinen aktuellen Rechten. Verlangt es Administratorrechte, startet Appmenu es nicht.")
}

fn cap_msi() -> Value {
    cap("limited", "Installationspakete verlangen oft Administratorrechte. Appmenu kann das nicht umgehen; Windows entscheidet.")
}

fn caps(elev: &str, winget: bool) -> Value {
    let install = if !winget {
        cap(
            "unavailable",
            "Der Windows-Paketdienst (winget) wurde auf diesem Gerät nicht gefunden.",
        )
    } else if elev == "unknown" {
        cap(
            "limited",
            "Rechte nicht sicher feststellbar. Installation nur für dein Konto, Ausgang ungewiss.",
        )
    } else {
        cap(
            "available",
            "Installation nur für dein Konto, ohne Administratorrechte.",
        )
    };
    json!({
        "search": cap("available", "Die Suche benötigt keine besonderen Rechte."),
        "install_user": install,
        "open_exe": cap_exe(),
        "open_msi": cap_msi(),
    })
}

fn get_path(state: &State<'_, AppState>, token: &str) -> Result<PathBuf, String> {
    let p = state
        .files
        .lock()
        .map_err(|_| "Interner Fehler.".to_string())?
        .get(token)
        .cloned()
        .ok_or_else(|| "Unbekannte Auswahl.".to_string())?;
    if !p.is_file() {
        return Err("Die Datei existiert nicht mehr.".into());
    }
    Ok(p)
}

#[tauri::command]
async fn info() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let elev = elevation();
        let w = winget_version();
        json!({
            "user": env_or("USERNAME"),
            "pc": env_or("COMPUTERNAME"),
            "elevation": elev,
            "winget": w.is_some(),
            "wingetVersion": w.clone().map(|v| format!("winget {v}")).unwrap_or_default(),
            "caps": caps(elev, w.is_some()),
        })
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn search(q: String) -> Result<Value, String> {
    let q = q.trim().to_string();
    if q.is_empty()
        || q.chars().count() > 80
        || q.chars().any(|c| c.is_control())
        || q.starts_with('-')
    {
        return Err("Ungültige Suche.".into());
    }
    tauri::async_runtime::spawn_blocking(move || -> Result<Value, String> {
        let out = hidden("winget")
            .args([
                "search",
                "--query",
                q.as_str(),
                "--source",
                "winget",
                "--accept-source-agreements",
            ])
            .output()
            .map_err(|_| "winget wurde nicht gefunden.".to_string())?;
        let lines = clean_lines(&String::from_utf8_lossy(&out.stdout));
        Ok(json!({ "items": parse_table(&lines) }))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn detail(id: String) -> Result<Value, String> {
    if !valid_id(&id) {
        return Err("Ungültige Paket-ID.".into());
    }
    tauri::async_runtime::spawn_blocking(move || -> Result<Value, String> {
        let out = hidden("winget")
            .args([
                "show",
                "--id",
                id.as_str(),
                "--exact",
                "--source",
                "winget",
                "--accept-source-agreements",
            ])
            .output()
            .map_err(|_| "winget wurde nicht gefunden.".to_string())?;
        let lines: Vec<String> = clean_lines(&String::from_utf8_lossy(&out.stdout))
            .into_iter()
            .take(45)
            .collect();
        if lines.is_empty() {
            return Err("Keine Details gefunden.".into());
        }
        Ok(json!({ "text": lines.join("\n") }))
    })
    .await
    .map_err(|e| e.to_string())?
}

fn push(job: &Arc<Mutex<Job>>, s: String) {
    if let Ok(mut j) = job.lock() {
        j.lines.push(s);
        if j.lines.len() > 2000 {
            j.lines.remove(0);
        }
    }
}

fn flush(acc: &mut Vec<u8>, job: &Arc<Mutex<Job>>) {
    if acc.is_empty() {
        return;
    }
    let s = String::from_utf8_lossy(acc).to_string();
    acc.clear();
    let t = s.trim();
    if t.chars().count() <= 1 || t.contains(['█', '▒', '▓', '░']) {
        return;
    }
    if let Ok(mut j) = job.lock() {
        if j.lines.last().is_some_and(|l| l == t) {
            return;
        }
        j.lines.push(t.to_string());
        if j.lines.len() > 2000 {
            j.lines.remove(0);
        }
    }
}

fn pump<R: Read + Send + 'static>(mut r: R, job: Arc<Mutex<Job>>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let mut acc: Vec<u8> = Vec::new();
        loop {
            let n = match r.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            for &b in &buf[..n] {
                if b == b'\r' || b == b'\n' {
                    flush(&mut acc, &job);
                } else {
                    acc.push(b);
                }
            }
        }
        flush(&mut acc, &job);
    })
}

fn run_install(ids: Vec<String>, job: Arc<Mutex<Job>>) {
    for id in ids {
        push(&job, format!("== {id} =="));
        let child = hidden("winget")
            .args([
                "install",
                "--id",
                id.as_str(),
                "--exact",
                "--scope",
                "user",
                "--source",
                "winget",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let code = match child {
            Ok(mut c) => {
                let h1 = c.stdout.take().map(|o| pump(o, job.clone()));
                let h2 = c.stderr.take().map(|o| pump(o, job.clone()));
                let code = c.wait().ok().and_then(|s| s.code()).unwrap_or(-1);
                if let Some(h) = h1 {
                    let _ = h.join();
                }
                if let Some(h) = h2 {
                    let _ = h.join();
                }
                code
            }
            Err(e) => {
                push(&job, format!("winget konnte nicht gestartet werden: {e}"));
                -1
            }
        };
        push(
            &job,
            if code == 0 {
                "OK".to_string()
            } else {
                format!("Fehler (Code {code})")
            },
        );
        if let Ok(mut j) = job.lock() {
            j.codes.push(code);
        }
    }
    if let Ok(mut j) = job.lock() {
        j.done = true;
    }
}

#[tauri::command]
async fn install(ids: Vec<String>, state: State<'_, AppState>) -> Result<Value, String> {
    if ids.is_empty() || ids.len() > 10 || !ids.iter().all(|i| valid_id(i)) {
        return Err("Keine gültige Auswahl.".into());
    }
    let has = tauri::async_runtime::spawn_blocking(winget_version)
        .await
        .map_err(|e| e.to_string())?;
    if has.is_none() {
        return Err("winget wurde nicht gefunden.".into());
    }
    let job = Arc::new(Mutex::new(Job::default()));
    let jid = new_id();
    state
        .jobs
        .lock()
        .map_err(|_| "Interner Fehler.".to_string())?
        .insert(jid.clone(), job.clone());
    std::thread::spawn(move || run_install(ids, job));
    Ok(json!({ "job": jid }))
}

#[tauri::command]
fn job(id: String, state: State<'_, AppState>) -> Result<Value, String> {
    let arc = state
        .jobs
        .lock()
        .map_err(|_| "Interner Fehler.".to_string())?
        .get(&id)
        .cloned()
        .ok_or_else(|| "Unbekannter Job.".to_string())?;
    let g = arc.lock().map_err(|_| "Interner Fehler.".to_string())?;
    let n = g.lines.len();
    let lines: Vec<&String> = g.lines.iter().skip(n.saturating_sub(80)).collect();
    let ok = g.done && !g.codes.is_empty() && g.codes.iter().all(|c| *c == 0);
    Ok(json!({ "lines": lines, "done": g.done, "ok": ok }))
}

#[tauri::command]
async fn scan(state: State<'_, AppState>) -> Result<Value, String> {
    let mut dirs: Vec<PathBuf> = vec![];
    if let Ok(p) = std::env::var("USERPROFILE") {
        dirs.push(Path::new(&p).join("Downloads"));
        dirs.push(Path::new(&p).join("Desktop"));
    }
    let found = tauri::async_runtime::spawn_blocking(move || {
        let mut out: Vec<(SystemTime, PathBuf)> = vec![];
        for d in dirs {
            if let Ok(rd) = std::fs::read_dir(&d) {
                for e in rd.flatten() {
                    let p = e.path();
                    let ext = ext_of(&p);
                    if (ext == "exe" || ext == "msi") && p.is_file() {
                        let m = e
                            .metadata()
                            .and_then(|m| m.modified())
                            .unwrap_or(SystemTime::UNIX_EPOCH);
                        out.push((m, p));
                    }
                }
            }
        }
        out.sort_by_key(|a| std::cmp::Reverse(a.0));
        out.truncate(60);
        out
    })
    .await
    .map_err(|e| e.to_string())?;
    let mut list = vec![];
    let mut map = state
        .files
        .lock()
        .map_err(|_| "Interner Fehler.".to_string())?;
    for (_, p) in found {
        let t = new_id();
        list.push(json!({
            "token": t,
            "name": p.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            "dir": p.parent().map(|d| d.display().to_string()).unwrap_or_default(),
            "kind": ext_of(&p),
        }));
        map.insert(t, p);
    }
    Ok(json!({ "files": list }))
}

#[tauri::command]
async fn pick(state: State<'_, AppState>) -> Result<Value, String> {
    let chosen = tauri::async_runtime::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Datei für Appmenu wählen")
            .add_filter("Programme", &["exe", "msi"])
            .add_filter("Alle Dateien", &["*"])
            .pick_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    match chosen {
        Some(p) => {
            let t = new_id();
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            state
                .files
                .lock()
                .map_err(|_| "Interner Fehler.".to_string())?
                .insert(t.clone(), p);
            Ok(json!({ "token": t, "name": name }))
        }
        None => Ok(json!({ "token": Value::Null })),
    }
}

fn classify(name: &str) -> (String, Value, Vec<String>) {
    let lower = name.to_lowercase();
    let mut warn = vec![];
    for d in [
        "jpg", "jpeg", "png", "gif", "pdf", "doc", "docx", "xls", "xlsx", "txt", "zip",
    ] {
        for e in ["exe", "msi", "bat", "cmd", "ps1"] {
            if lower.ends_with(&format!(".{d}.{e}")) {
                warn.push(
                    "Doppelte Dateiendung: Die Datei tarnt sich möglicherweise als anderer Typ."
                        .to_string(),
                );
            }
        }
    }
    let ext = lower.rsplit('.').next().unwrap_or("");
    match ext {
        "exe" => ("Programm (EXE)".to_string(), cap_exe(), warn),
        "msi" => (
            "Windows-Installationspaket (MSI)".to_string(),
            cap_msi(),
            warn,
        ),
        "ps1" | "bat" | "cmd" | "vbs" | "js" | "wsf" => (
            "Skript".to_string(),
            cap("unavailable", "Skripte führt Appmenu nicht aus."),
            warn,
        ),
        _ => (
            "Nicht unterstützt".to_string(),
            cap("unavailable", "Appmenu öffnet diesen Dateityp nicht."),
            warn,
        ),
    }
}

#[tauri::command]
async fn file_info(token: String, state: State<'_, AppState>) -> Result<Value, String> {
    let p = get_path(&state, &token)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<Value, String> {
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let (kind, capv, warnings) = classify(&name);
        let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        #[cfg(windows)]
        let motw = std::fs::File::open(format!("{}:Zone.Identifier", p.display())).is_ok();
        #[cfg(not(windows))]
        let motw = false;
        Ok(json!({
            "name": name,
            "dir": p.parent().map(|d| d.display().to_string()).unwrap_or_default(),
            "size": size,
            "kind": kind,
            "cap": capv,
            "warnings": warnings,
            "motw": motw,
        }))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn hash(token: String, state: State<'_, AppState>) -> Result<Value, String> {
    let p = get_path(&state, &token)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<Value, String> {
        let mut f = std::fs::File::open(&p).map_err(|e| e.to_string())?;
        let mut h = Sha256::new();
        let mut buf = vec![0u8; 1 << 20];
        loop {
            let n = f.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            h.update(&buf[..n]);
        }
        Ok(json!({ "sha256": hex(&h.finalize()) }))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn open(token: String, state: State<'_, AppState>) -> Result<Value, String> {
    let p = get_path(&state, &token)?;
    let ext = ext_of(&p);
    let spawned = match ext.as_str() {
        "exe" => Command::new(&p)
            .current_dir(p.parent().unwrap_or_else(|| Path::new(".")))
            .spawn(),
        "msi" => Command::new("msiexec").arg("/i").arg(&p).spawn(),
        _ => return Ok(json!({ "message": "Appmenu öffnet diesen Dateityp nicht." })),
    };
    let message = match spawned {
        Ok(_) => {
            if ext == "msi" {
                "Installer geöffnet. Windows entscheidet über die Rechte.".to_string()
            } else {
                "Programm gestartet.".to_string()
            }
        }
        Err(e) => match e.raw_os_error() {
            Some(740) => "Das Programm verlangt Administratorrechte. Appmenu startet es nicht und umgeht das nicht.".to_string(),
            Some(1260) => "Eine Gerät/ Richtlinie blockiert dieses Programm. Appmenu umgeht das nicht.".to_string(),
            _ => format!("Konnte nicht gestartet werden: {e}"),
        },
    };
    Ok(json!({ "message": message }))
}

#[tauri::command]
async fn ai_models() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(|| -> Result<Value, String> {
        let response = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|_| "Ollama-Client konnte nicht erstellt werden.".to_string())?
            .get("http://127.0.0.1:11434/api/tags")
            .send()
            .map_err(|_| "Ollama ist nicht erreichbar.".to_string())?;

        if !response.status().is_success() {
            return Err("Ollama hat einen Fehler gemeldet.".into());
        }

        let body = response
            .text()
            .map_err(|_| "Ollama-Antwort konnte nicht gelesen werden.".to_string())?;
        let models = parse_ollama_tags(&body)?;
        Ok(json!({ "models": models }))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn ai_recommend(available_ram_mb: Option<u64>) -> Result<Value, String> {
    let models = ai_models().await?;
    let installed = models
        .get("models")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(recommend_model(available_ram_mb, &installed))
}

#[tauri::command]
async fn ai_validate(raw: String, sources: Vec<Value>) -> Result<Value, String> {
    if sources.len() > 16 {
        return Err("Zu viele Quellen.".into());
    }

    let trusted = sources
        .into_iter()
        .map(|source| {
            let id = source
                .get("id")
                .and_then(Value::as_u64)
                .and_then(|id| u32::try_from(id).ok())
                .ok_or("Ungültige Quellen-ID.")?;
            let title = source
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let url = source
                .get("url")
                .and_then(Value::as_str)
                .ok_or("Ungültige Quellen-URL.")?
                .to_string();
            let text = source
                .get("text")
                .and_then(Value::as_str)
                .ok_or("Ungültiger Quellentext.")?
                .to_string();

            Ok(TrustedSource {
                id,
                title,
                url,
                text,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    validate_quotes(&raw, &trusted)
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            info, search, detail, install, job, scan, pick, file_info, hash, open,
            ai_models, ai_recommend, ai_validate
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Start von Appmenu");
}
