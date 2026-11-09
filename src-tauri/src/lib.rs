use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tauri::State;

mod ai;
mod packages;
mod safety;
mod terminal;

use ai::{parse_ollama_tags, recommend_model, validate_quotes, TrustedSource};
use packages::{
    clean_lines, parse_table, preflight_user_installer, user_installer_args, valid_id,
    winget_command, winget_failure, winget_version,
};

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
    known_packages: Mutex<HashSet<String>>,
    installing: Arc<AtomicBool>,
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

fn validate_selection(ids: &[String], known: &HashSet<String>) -> Result<(), String> {
    if ids.is_empty() || ids.len() > 10 || !ids.iter().all(|id| valid_id(id)) {
        return Err("Keine gültige Auswahl.".into());
    }
    let unique: HashSet<&String> = ids.iter().collect();
    if unique.len() != ids.len() {
        return Err("Doppelte Paket-IDs in der Auswahl.".into());
    }
    if !ids.iter().all(|id| known.contains(id)) {
        return Err("Pakete müssen aus einer erfolgreichen Suche dieser Sitzung stammen.".into());
    }
    Ok(())
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

fn elevation() -> &'static str {
    safety::elevation()
}

fn cap(state: &str, reason: &str) -> Value {
    json!({ "state": state, "reason": reason })
}

fn cap_exe() -> Value {
    if safety::execution_allowed(elevation()) {
        cap("limited", "Start nach Bestätigung mit deinen aktuellen Rechten. Keine Signaturprüfung; Windows kann den Start blockieren.")
    } else {
        cap(
            "unavailable",
            "Erhöhte oder unbekannte Prozessrechte: Bitte Appmenu normal neu starten.",
        )
    }
}

fn cap_msi() -> Value {
    cap("unavailable", "Direkter MSI-Start ist deaktiviert. Nutze eine unterstützte Benutzerinstallation über die Suche.")
}

fn caps(elev: &str, winget: bool) -> Value {
    let install = if !safety::execution_allowed(elev) {
        cap(
            "unavailable",
            "Erhöhte oder unbekannte Prozessrechte: Bitte Appmenu normal neu starten.",
        )
    } else if !winget {
        cap(
            "unavailable",
            "Der Windows-Paketdienst (winget) wurde auf diesem Gerät nicht gefunden.",
        )
    } else {
        cap("limited", "Nur Benutzerinstallationen. Paketunterstützung wird geprüft; Geräterichtlinien können die Ausführung trotzdem verhindern.")
    };
    let search = if winget {
        cap("available", "Programmsuche über winget.")
    } else {
        cap(
            "unavailable",
            "Ohne winget ist die Programmsuche nicht verfügbar.",
        )
    };
    json!({
        "search": search,
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
    std::fs::canonicalize(p).map_err(|_| "Die Datei konnte nicht aufgelöst werden.".to_string())
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
async fn search(q: String, state: State<'_, AppState>) -> Result<Value, String> {
    let q = q.trim().to_string();
    if q.is_empty()
        || q.chars().count() > 80
        || q.chars().any(|c| c.is_control())
        || q.starts_with('-')
    {
        return Err("Ungültige Suche.".into());
    }
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<Value, String> {
        let out = winget_command()?
            .args([
                "search",
                "--query",
                q.as_str(),
                "--source",
                "winget",
                "--accept-source-agreements",
                "--disable-interactivity",
            ])
            .output()
            .map_err(|e| format!("winget konnte nicht gestartet werden: {e}"))?;
        if !out.status.success() {
            if out.status.code().map(|c| c as u32) == Some(0x8a15_0014) {
                return Ok(json!({ "items": [] }));
            }
            return Err(winget_failure("Suche", &out));
        }
        let lines = clean_lines(&String::from_utf8_lossy(&out.stdout));
        let items = parse_table(&lines);
        if items.is_empty() {
            return Err("Die winget-Ausgabe konnte nicht zuverlässig gelesen werden.".into());
        }
        Ok(json!({ "items": items }))
    })
    .await
    .map_err(|e| e.to_string())??;
    let found: Vec<String> = result["items"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item["id"].as_str().map(str::to_string))
        .collect();
    let mut known = state
        .known_packages
        .lock()
        .map_err(|_| "Interner Fehler.".to_string())?;
    if known.len() + found.iter().filter(|id| !known.contains(*id)).count() > 512 {
        return Err("Zu viele Pakete in dieser Sitzung. Bitte Appmenu neu starten.".into());
    }
    known.extend(found);
    Ok(result)
}

#[tauri::command]
async fn detail(id: String) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        preflight_user_installer(&id).map(|text| json!({ "text": text }))
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
                    if acc.len() >= 16 * 1024 {
                        flush(&mut acc, &job);
                    }
                }
            }
        }
        flush(&mut acc, &job);
    })
}

fn run_install(ids: Vec<String>, job: Arc<Mutex<Job>>) {
    for id in ids {
        push(&job, format!("== {id} =="));
        let mut command = match safety::require_normal_process().and_then(|_| winget_command()) {
            Ok(command) => command,
            Err(error) => {
                push(&job, error);
                if let Ok(mut j) = job.lock() {
                    j.codes.push(-1);
                }
                break;
            }
        };
        let child = command
            .args(user_installer_args("install", &id))
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
    {
        let known = state
            .known_packages
            .lock()
            .map_err(|_| "Interner Fehler.".to_string())?;
        validate_selection(&ids, &known)?;
    }
    safety::require_normal_process()?;
    let guard = safety::InstallGuard::acquire(state.installing.clone())?;
    let ids = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<String>, String> {
        // Check the entire plan before starting any installation.
        safety::require_normal_process()?;
        for id in &ids {
            preflight_user_installer(id)?;
        }
        Ok(ids)
    })
    .await
    .map_err(|e| e.to_string())??;
    let job = Arc::new(Mutex::new(Job::default()));
    let jid = new_id();
    state
        .jobs
        .lock()
        .map_err(|_| "Interner Fehler.".to_string())?
        .insert(jid.clone(), job.clone());
    std::thread::Builder::new()
        .name("appmenu-install".into())
        .spawn(move || {
            let _guard = guard;
            run_install(ids, job);
        })
        .map_err(|e| format!("Installation konnte nicht gestartet werden: {e}"))?;
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
    safety::require_normal_process()?;
    let p = get_path(&state, &token)?;
    let ext = ext_of(&p);
    let spawned = match ext.as_str() {
        "exe" => Command::new(&p)
            .current_dir(p.parent().unwrap_or_else(|| Path::new(".")))
            .spawn(),
        "msi" => return Err("Direkter MSI-Start ist in Appmenu Lite deaktiviert.".into()),
        _ => return Ok(json!({ "message": "Appmenu öffnet diesen Dateityp nicht." })),
    };
    let message = match spawned {
        Ok(mut child) => {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            "Windows hat den Prozessstart angenommen. Die Datei wurde nicht auf Schadsoftware geprüft."
                .to_string()
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

#[tauri::command]
fn cli_terminal_status(app: tauri::AppHandle) -> Value {
    terminal::status(&app)
}

#[tauri::command]
fn open_cli_terminal(app: tauri::AppHandle) -> Result<Value, String> {
    terminal::launch(&app)
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            info,
            search,
            detail,
            install,
            job,
            scan,
            pick,
            file_info,
            hash,
            open,
            ai_models,
            ai_recommend,
            ai_validate,
            cli_terminal_status,
            open_cli_terminal
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Start von Appmenu");
}

#[cfg(test)]
mod safety_regressions {
    use super::*;

    #[test]
    fn ids_reject_options_and_shell_fragments() {
        for id in ["7zip.7zip", "Mozilla.Firefox", "Notepad++.Notepad++"] {
            assert!(valid_id(id));
        }
        for id in ["", "--scope", "a b", "a;b", "a&b", "../x", "a\nb"] {
            assert!(!valid_id(id));
        }
    }

    #[test]
    fn selection_requires_discovered_unique_ids_and_respects_limit() {
        let known = HashSet::from(["Vendor.App".to_string(), "Vendor.Other".to_string()]);
        assert!(validate_selection(&["Vendor.App".into()], &known).is_ok());
        assert!(validate_selection(&["Vendor.Other".into(), "Vendor.App".into()], &known).is_ok());
        assert!(validate_selection(&[], &known).is_err());
        assert!(validate_selection(&["Unseen.App".into()], &known).is_err());
        assert!(validate_selection(&["Vendor.App".into(), "Vendor.App".into()], &known).is_err());
        let many: Vec<String> = (0..11).map(|i| format!("Vendor.App{i}")).collect();
        let all = many.iter().cloned().collect();
        assert!(validate_selection(&many, &all).is_err());
    }

    #[test]
    fn installer_and_preflight_share_fixed_user_scope() {
        for action in ["show", "install"] {
            let args = user_installer_args(action, "Vendor.App");
            assert!(args.windows(2).any(|a| a == ["--scope", "user"]));
            assert!(args.windows(2).any(|a| a == ["--source", "winget"]));
            assert!(args.windows(2).any(|a| a == ["--id", "Vendor.App"]));
            assert!(args.contains(&"--disable-interactivity"));
            assert!(!args.contains(&"machine"));
            assert_eq!(
                args.contains(&"--accept-package-agreements"),
                action == "install"
            );
        }
    }

    #[test]
    fn capabilities_fail_closed_and_do_not_promise_policy_approval() {
        for mode in ["unknown", "elevated"] {
            assert_eq!(caps(mode, true)["install_user"]["state"], "unavailable");
        }
        assert_eq!(caps("standard", true)["install_user"]["state"], "limited");
        assert_eq!(caps("standard", false)["search"]["state"], "unavailable");
        assert_eq!(cap_msi()["state"], "unavailable");
    }

    #[test]
    fn table_parser_preserves_unicode_names_and_validates_ids() {
        let lines = vec![
            "Name        Id               Version  Source".into(),
            "---------------------------------------------".into(),
            "Äpfel       Vendor.App       1.0      winget".into(),
            "Bad         --scope          2.0      winget".into(),
        ];
        let items = parse_table(&lines);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["nm"], "Äpfel");
        assert_eq!(items[0]["id"], "Vendor.App");
        assert!(parse_table(&["not a winget table".into()]).is_empty());
    }
}
