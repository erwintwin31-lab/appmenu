//! Headless interface to the real Appmenu backend, not an arbitrary shell wrapper.
#[path = "../../src-tauri/src/ai.rs"]
mod ai;
#[path = "../../src-tauri/src/packages.rs"]
mod packages;
#[path = "../../src-tauri/src/safety.rs"]
mod safety;
use clap::{Args, Parser, Subcommand};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, io::{self, BufRead, IsTerminal, Read, Write}, path::{Path, PathBuf}, process::{Command, ExitStatus, Stdio}, sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc, OnceLock}, time::{Duration, Instant, SystemTime}};
const MAX_TEXT: u64 = 2 * 1024 * 1024;
const MAX_CAPTURE: usize = 2 * 1024 * 1024;
static INSTALLING: OnceLock<Arc<AtomicBool>> = OnceLock::new();
#[derive(Debug)]
pub struct Failure { pub code: u8, pub message: String }
impl Failure {
    pub fn error(s: impl Into<String>) -> Self { Self { code: 1, message: s.into() } }
    pub fn usage(s: impl Into<String>) -> Self { Self { code: 2, message: s.into() } }
}
type Result<T> = std::result::Result<T, Failure>;
fn io_error(e: io::Error) -> Failure { Failure::error(e.to_string()) }
#[derive(Debug, Parser)]
#[command(name = "appmenu-cli", version, about = "Appmenu ohne GUI: Pakete suchen, Dateien prüfen und bewusst als Benutzer installieren.", after_help = "Ohne Unterbefehl: interaktives Menü (nur im Terminal). Keine Rechteerhöhung oder beliebigen Shell-Befehle. Exitcodes: 0 Erfolg, 1 Fehler/Blockierung, 2 Aufruf, 124 Zeitlimit, 130 Abbruch.")]
pub struct Cli {
    #[arg(long, global = true, help = "Ein JSON-Dokument auf stdout; Fortschritt und Rückfragen auf stderr")]
    pub json: bool,
    #[arg(long, global=true, default_value_t=120, value_parser=clap::value_parser!(u64).range(1..=7200), help="Zeitlimit je externem Prozess in Sekunden")]
    pub timeout: u64,
    #[command(subcommand)]
    pub command: Option<Action>,
}
#[derive(Debug, Subcommand)]
pub enum Action {
    /// Plattform, Prozessrechte und Funktionsumfang.
    Info,
    /// Programme in der offiziellen WinGet-Quelle suchen (Windows).
    Search { query: String },
    /// Exakte Paket-ID auf Benutzerinstallation prüfen.
    #[command(alias = "show")]
    Detail { id: String },
    /// Erst Plan prüfen, dann nach Zustimmung als Benutzer installieren.
    Install(Install),
    /// EXE/MSI auflisten; nicht rekursiv, nichts ausführen.
    Scan { #[arg(long)] dir: Option<PathBuf>, #[arg(long, default_value_t=60, value_parser=clap::value_parser!(u16).range(1..=200))] limit: u16 },
    /// Dateimetadaten, Endungswarnungen und Startfähigkeit.
    FileInfo { path: PathBuf },
    /// SHA-256 einer regulären Datei oder von stdin ("-").
    Hash { path: PathBuf },
    /// EXE direkt unter Windows starten; keine MSI/Skripte/Rechteerhöhung.
    Open { path: PathBuf, #[arg(long)] yes: bool },
    /// Vorhandene lokale KI-Helfer; keine generative Chatfunktion.
    Ai { #[command(subcommand)] command: AiAction },
    /// Interaktives Menü; leere Eingabe führt zurück.
    Menu,
}
#[derive(Debug, Args)]
pub struct Install {
    #[arg(required=true, num_args=1..=10)] pub ids: Vec<String>,
    #[arg(long)] pub dry_run: bool,
    #[arg(long, help = "Installation und Paketbedingungen ausdrücklich bestätigen")] pub yes: bool,
}
#[derive(Debug, Subcommand)]
pub enum AiAction {
    /// Installierte Modelle vom lokalen Ollama abfragen.
    Models,
    /// Konservativer Vorschlag mit explizit angegebenem freien RAM.
    Recommend { #[arg(long, value_parser=clap::value_parser!(u64).range(1..=1048576))] available_ram_mb: Option<u64> },
    /// Zitate gegen bereitgestellte JSON-Quellen prüfen; keine Faktenprüfung.
    Validate { #[arg(long)] response: PathBuf, #[arg(long)] sources: PathBuf },
}
pub fn terminal_text(s: &str) -> String {
    s.chars().filter(|c| (!c.is_control() || matches!(c,'\n'|'\t')) && !matches!(c,'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}'|'\u{200e}'|'\u{200f}')).collect()
}
struct Context { cancel: Arc<AtomicBool>, timeout: Duration }
impl Context {
    fn check(&self) -> Result<()> {
        if self.cancel.load(Ordering::Acquire) { Err(Failure { code: 130, message: "Abgebrochen; keine Rückabwicklung bereits erfolgter Schritte.".into() }) } else { Ok(()) }
    }
}
pub fn execute(cli: &Cli, cancel: Arc<AtomicBool>) -> Result<Value> {
    let ctx = Context { cancel, timeout: Duration::from_secs(cli.timeout) };
    ctx.check()?;
    match cli.command.as_ref().ok_or_else(|| Failure::usage("Unterbefehl fehlt; --help oder menu verwenden."))? {
        Action::Info => information(&ctx),
        Action::Search { query } => search(query, &ctx),
        Action::Detail { id } => detail(id, &ctx),
        Action::Install(args) => install(args, &ctx),
        Action::Scan { dir, limit } => scan(dir.as_deref(), *limit),
        Action::FileInfo { path } => file_info(path),
        Action::Hash { path } => hash_file(path, &ctx),
        Action::Open { path, yes } => open_file(path, *yes, &ctx),
        Action::Ai { command } => { let result = ai_action(command); ctx.check()?; result }
        Action::Menu => Err(Failure::usage("Menü nur über den interaktiven Einstiegspunkt.")),
    }
}
struct Captured { status: ExitStatus, stdout: String, stderr: String }
// Bounded simultaneous pipe readers prevent child-process deadlocks and unbounded capture.
fn run_command(cmd: &mut Command, ctx: &Context, progress: bool) -> Result<Captured> {
    ctx.check()?;
    let mut child = cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(io_error)?;
    let (tx, rx) = mpsc::sync_channel::<(bool, Vec<u8>)>(16);
    fn pump(mut r: impl Read + Send + 'static, is_err: bool, tx: mpsc::SyncSender<(bool, Vec<u8>)>) {
        std::thread::spawn(move || {
            let mut buf = [0; 4096];
            loop {
                match r.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => { if tx.send((is_err, buf[..n].to_vec())).is_err() { break; } }
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
        });
    }
    if let Some(r) = child.stdout.take() { pump(r, false, tx.clone()) }
    if let Some(r) = child.stderr.take() { pump(r, true, tx.clone()) }
    drop(tx);
    let start = Instant::now();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut status = None;
    let mut drained = false;
    loop {
        let problem = if ctx.cancel.load(Ordering::Acquire) { Some((130, "Abgebrochen")) } else if start.elapsed() > ctx.timeout { Some((124, "Zeitlimit erreicht")) } else { None };
        if let Some((code, message)) = problem {
            let _ = child.kill(); let _ = child.wait();
            return Err(Failure {code,message:format!("{message}; bereits gestartete Installer können weiterlaufen. Keine Rückabwicklung.")});
        }
        match rx.recv_timeout(Duration::from_millis(40)) {
            Ok((is_err, bytes)) => {
                if out.len() + err.len() + bytes.len() > MAX_CAPTURE {
                    let _ = child.kill(); let _ = child.wait();
                    return Err(Failure::error("Prozessausgabe überschreitet 2 MiB; abgebrochen."));
                }
                if progress { eprint!("{}", terminal_text(&String::from_utf8_lossy(&bytes))); }
                if is_err { err.extend(bytes) } else { out.extend(bytes) }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => { drained = true; std::thread::sleep(Duration::from_millis(10)); }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if status.is_none() {
            match child.try_wait() { Ok(s) => status = s, Err(e) => { let _ = child.kill(); let _ = child.wait(); return Err(io_error(e)); } }
        }
        if drained && status.is_some() { break; }
    }
    Ok(Captured { status: status.ok_or_else(|| Failure::error("Prozessstatus fehlt."))?, stdout: String::from_utf8_lossy(&out).into_owned(), stderr: String::from_utf8_lossy(&err).into_owned() })
}
fn winget(args: &[String], ctx: &Context, progress: bool) -> Result<Captured> {
    let mut cmd = packages::winget_command().map_err(Failure::error)?; cmd.args(args); run_command(&mut cmd, ctx, progress)
}
fn strings(args: &[&str]) -> Vec<String> { args.iter().map(|s| (*s).to_owned()).collect() }
fn require_success(r: Captured, action: &str) -> Result<Captured> {
    if r.status.success() { Ok(r) } else { Err(Failure::error(format!("{action} fehlgeschlagen (Code {}). {}", r.status.code().unwrap_or(-1), terminal_text(&format!("{}\n{}", r.stderr, r.stdout)).chars().take(2000).collect::<String>()))) }
}
fn information(ctx: &Context) -> Result<Value> {
    let version = if cfg!(windows) { winget(&strings(&["--version"]), ctx, false).ok().filter(|r| r.status.success()).map(|r| terminal_text(r.stdout.trim())) } else { None };
    ctx.check()?;
    let normal = safety::execution_allowed(safety::elevation());
    Ok(json!({"platform":std::env::consts::OS,"version":env!("CARGO_PKG_VERSION"),"elevation":safety::elevation(),"wingetVersion":version,"capabilities":{"search":version.is_some(),"install_user":cfg!(windows)&&normal&&version.is_some(),"open_exe":cfg!(windows)&&normal,"open_msi":false,"scripts":false,"scan":true,"hash":true,"file_info":true,"ai_local":true},"notes":["Keine Rechteerhöhung oder Richtlinienumgehung.","Linux: keine Paketinstallation oder EXE-Ausführung.","Keine Sandbox oder Schadsoftwareprüfung."]}))
}
fn validate_query(q: &str) -> Result<&str> {
    let q = q.trim();
    if q.is_empty() || q.chars().count() > 80 || q.starts_with('-') || q.chars().any(char::is_control) { Err(Failure::usage("Suche: 1 bis 80 Zeichen, keine Steuerzeichen, kein führendes Minus.")) } else { Ok(q) }
}
fn search(query: &str, ctx: &Context) -> Result<Value> {
    let q = validate_query(query)?;
    let r = winget(&strings(&["search", "--query", q, "--source", "winget", "--accept-source-agreements", "--disable-interactivity"]), ctx, false)?;
    if r.status.code().map(|c| c as u32) == Some(0x8a15_0014) { return Ok(json!({"items":[]})); }
    let r = require_success(r, "Suche")?;
    let items = packages::parse_table(&packages::clean_lines(&r.stdout));
    if items.is_empty() { return Err(Failure::error("WinGet-Ausgabe nicht zuverlässig lesbar.")); }
    Ok(json!({"items":items}))
}
fn detail(id: &str, ctx: &Context) -> Result<Value> {
    if !packages::valid_id(id) { return Err(Failure::usage("Ungültige exakte Paket-ID.")); }
    let args = packages::user_installer_args("show", id).into_iter().map(str::to_owned).collect::<Vec<_>>();
    let r = require_success(winget(&args, ctx, false)?, "Prüfung der Benutzerinstallation")?;
    let lines: Vec<_> = packages::clean_lines(&r.stdout).into_iter().take(45).map(|s| terminal_text(&s)).collect();
    if lines.is_empty() { return Err(Failure::error("Keine lesbaren Paketdetails.")); }
    Ok(json!({"id":id,"scope":"user","text":lines.join("\n")}))
}
fn validate_ids(ids: &[String]) -> Result<()> {
    let distinct: HashSet<_> = ids.iter().map(|s| s.to_ascii_lowercase()).collect();
    if ids.is_empty() || ids.len() > 10 || !ids.iter().all(|s| packages::valid_id(s)) || distinct.len() != ids.len() { Err(Failure::usage("1 bis 10 gültige, unterschiedliche exakte Paket-IDs erforderlich.")) } else { Ok(()) }
}
fn read_input(ctx: &Context) -> Result<Option<String>> {
    let (tx, rx) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut s = String::new();
        let r = io::stdin().lock().take(8193).read_line(&mut s).map(|n| (n, s));
        let _ = tx.send(r);
    });
    loop {
        ctx.check()?;
        match rx.recv_timeout(Duration::from_millis(40)) {
            Ok(Ok((0, _))) => return Ok(None),
            Ok(Ok((n, _))) if n > 8192 => return Err(Failure::usage("Eingabe zu lang.")),
            Ok(Ok((_, s))) => return Ok(Some(s.trim().to_owned())),
            Ok(Err(e)) => return Err(io_error(e)),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => return Err(Failure::error("Terminaleingabe unterbrochen.")),
        }
    }
}
fn confirm(message: &str, yes: bool, ctx: &Context) -> Result<()> {
    ctx.check()?;
    if yes { return Ok(()); }
    if !io::stdin().is_terminal() { return Err(Failure::usage("Nichtinteraktiv: --yes ausdrücklich setzen oder --dry-run verwenden.")); }
    eprint!("{}\nExakt JA zum Bestätigen, sonst Abbruch: ", terminal_text(message));
    io::stderr().flush().map_err(io_error)?;
    if read_input(ctx)?.as_deref() == Some("JA") { Ok(()) } else { Err(Failure { code: 130, message: "Nicht bestätigt; nichts gestartet.".into() }) }
}
fn install(args: &Install, ctx: &Context) -> Result<Value> {
    validate_ids(&args.ids)?;
    safety::require_normal_process().map_err(Failure::error)?;
    let _guard = safety::InstallGuard::acquire(INSTALLING.get_or_init(|| Arc::new(AtomicBool::new(false))).clone()).map_err(Failure::error)?;
    let mut plan = Vec::new();
    for id in &args.ids { ctx.check()?; plan.push(detail(id, ctx)?); }
    let commands: Vec<Vec<String>> = args.ids.iter().map(|id| packages::user_installer_args("install", id).into_iter().chain(std::iter::once("--silent")).map(str::to_owned).collect()).collect();
    if args.dry_run { return Ok(json!({"dry_run":true,"scope":"user","plan":plan,"arguments":commands,"source_agreements_accepted":true,"package_agreements_accepted":false})); }
    eprintln!("Installationsplan:\n{}", terminal_text(&serde_json::to_string_pretty(&plan).map_err(|e| Failure::error(e.to_string()))?));
    confirm("Diese Benutzerinstallationen starten und ihre Paket-/Lizenzbedingungen annehmen? Quellenbedingungen werden bei Suche/Prüfung bereits akzeptiert.",args.yes,ctx)?;
    let mut results = Vec::new();
    for (id, command) in args.ids.iter().zip(&commands) {
        ctx.check()?;
        safety::require_normal_process().map_err(Failure::error)?;
        eprintln!("\n== {} ==", terminal_text(id));
        let r = winget(command, ctx, true)?;
        if !r.status.success() { return Err(Failure::error(format!("Installation {id} fehlgeschlagen (Code {}). Weitere Pakete werden nicht gestartet; bisherige Installationen bleiben bestehen.",r.status.code().unwrap_or(-1)))); }
        results.push(json!({"id":id,"ok":true,"code":r.status.code()}));
    }
    Ok(json!({"scope":"user","results":results}))
}
fn resolved_file(p: &Path) -> Result<PathBuf> {
    let p = std::fs::canonicalize(p).map_err(io_error)?;
    if !std::fs::metadata(&p).map_err(io_error)?.is_file() { return Err(Failure::usage("Keine reguläre Datei.")); }
    Ok(p)
}
fn extension(p: &Path) -> String { p.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase() }
fn scan(dir: Option<&Path>, limit: u16) -> Result<Value> {
    let dirs = if let Some(dir) = dir {
        let p = std::fs::canonicalize(dir).map_err(io_error)?;
        if !p.is_dir() { return Err(Failure::usage("--dir muss einen Ordner bezeichnen.")); }
        vec![p]
    } else {
        let p = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from).ok_or_else(|| Failure::error("Benutzerordner fehlt; --dir angeben."))?;
        vec![p.join("Downloads"), p.join("Desktop")]
    };
    let mut files = Vec::new();
    let mut warnings = Vec::new();
    let mut inspected = 0;
    for dir in dirs {
        let entries = match std::fs::read_dir(&dir) { Ok(e) => e, Err(e) => { warnings.push(format!("{}: {e}", dir.display())); continue; } };
        for e in entries {
            inspected += 1;
            if inspected > 20000 { return Err(Failure::error("Mehr als 20.000 Einträge; kleineren Ordner wählen.")); }
            let e = e.map_err(io_error)?;
            if !e.file_type().map_err(io_error)?.is_file() { continue; }
            let path = e.path();
            let kind = extension(&path);
            if !matches!(kind.as_str(), "exe" | "msi") { continue; }
            let m = e.metadata().map_err(io_error)?;
            files.push((m.modified().unwrap_or(SystemTime::UNIX_EPOCH),json!({"path":path,"name":e.file_name().to_string_lossy(),"kind":kind,"size":m.len()})));
            if files.len() > limit as usize { files.sort_by_key(|x| std::cmp::Reverse(x.0)); files.truncate(limit as usize); }
        }
    }
    files.sort_by_key(|x| std::cmp::Reverse(x.0));
    Ok(json!({"files":files.into_iter().map(|x|x.1).collect::<Vec<_>>(),"warnings":warnings,"recursive":false}))
}
fn file_info(path: &Path) -> Result<Value> {
    let p = resolved_file(path)?;
    let name = p.file_name().unwrap_or_default().to_string_lossy();
    let kind = extension(&p);
    let mut warnings = Vec::new();
    let lower = name.to_lowercase();
    for disguise in ["jpg", "jpeg", "png", "gif", "pdf", "doc", "docx", "xls", "xlsx", "txt", "zip"] {
        for exe in ["exe", "msi", "bat", "cmd", "ps1"] {
            if lower.ends_with(&format!(".{disguise}.{exe}")) { warnings.push("Doppelte Dateiendung: mögliche Tarnung."); }
        }
    }
    #[cfg(windows)]
    let motw = { let mut ads = p.as_os_str().to_os_string(); ads.push(":Zone.Identifier"); std::fs::File::open(PathBuf::from(ads)).is_ok() };
    #[cfg(not(windows))]
    let motw = false;
    Ok(json!({"path":p,"name":name,"kind":kind,"size":std::fs::metadata(&p).map_err(io_error)?.len(),"warnings":warnings,"motw":motw,"can_open":cfg!(windows)&&kind=="exe"&&safety::execution_allowed(safety::elevation()),"malware_checked":false,"signature_checked":false,"reason":"Nur EXE unter Windows, nach Zustimmung und mit nicht erhöhten Prozessrechten. Keine Sandbox. MSI/Skripte bleiben gesperrt."}))
}
fn digest(mut reader: impl Read, ctx: &Context) -> Result<String> {
    let mut h = Sha256::new(); let mut buf = [0; 65536];
    loop { ctx.check()?; let n = reader.read(&mut buf).map_err(io_error)?; if n == 0 { break; } h.update(&buf[..n]); }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}
fn hash_stdin(ctx: &Context) -> Result<String> {
    let (tx, rx) = mpsc::sync_channel::<io::Result<Vec<u8>>>(4);
    std::thread::spawn(move || {
        let mut input = io::stdin().lock(); let mut buf = [0; 65536];
        loop {
            match input.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => { if tx.send(Ok(buf[..n].to_vec())).is_err() { break; } }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => { let _ = tx.send(Err(e)); break; }
            }
        }
    });
    let mut h = Sha256::new();
    loop {
        ctx.check()?;
        match rx.recv_timeout(Duration::from_millis(40)) { Ok(Ok(bytes)) => h.update(bytes), Ok(Err(e)) => return Err(io_error(e)), Err(mpsc::RecvTimeoutError::Timeout) => {}, Err(_) => break }
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}
fn hash_file(path: &Path, ctx: &Context) -> Result<Value> {
    if path == Path::new("-") {
        if io::stdin().is_terminal() { return Err(Failure::usage("hash - benötigt Daten über stdin.")); }
        return Ok(json!({"path":"-","sha256":hash_stdin(ctx)?}));
    }
    let p = resolved_file(path)?;
    Ok(json!({"path":p,"sha256":digest(std::fs::File::open(&p).map_err(io_error)?,ctx)?}))
}
fn open_file(path: &Path, yes: bool, ctx: &Context) -> Result<Value> {
    if !cfg!(windows) { return Err(Failure::error("EXE-Start nur unter Windows. Keine Wine-/Shell-Ausführung.")); }
    safety::require_normal_process().map_err(Failure::error)?;
    let p = resolved_file(path)?;
    if extension(&p) != "exe" { return Err(Failure::usage("Nur EXE-Dateien; keine MSI oder Skripte.")); }
    let before = std::fs::metadata(&p).map_err(io_error)?;
    confirm(&format!("Ausführen: {}\nKeine Signatur-/Schadsoftwareprüfung oder Sandbox. Das Programm kann Benutzerdateien verändern.",p.display()),yes,ctx)?;
    ctx.check()?;
    safety::require_normal_process().map_err(Failure::error)?;
    let after = std::fs::metadata(&p).map_err(io_error)?;
    if before.len() != after.len() || before.modified().ok() != after.modified().ok() { return Err(Failure::error("Datei während der Bestätigung verändert; nichts gestartet.")); }
    let mut child = Command::new(&p).current_dir(p.parent().ok_or_else(|| Failure::usage("Ordner fehlt."))?).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|e| match e.raw_os_error() { Some(740) => Failure::error("Programm verlangt Adminrechte; keine Rechteerhöhung."), Some(1260) => Failure::error("Geräterichtlinie blockiert den Start; keine Umgehung."), _ => io_error(e) })?;
    let pid = child.id();
    std::thread::spawn(move || { let _ = child.wait(); });
    Ok(json!({"started":true,"path":p,"pid":pid,"warning":"Start angenommen; weder Sicherheit noch erfolgreiche Installation nachgewiesen."}))
}
fn bounded_text(path: &Path) -> Result<String> {
    let mut s = String::new();
    std::fs::File::open(resolved_file(path)?).map_err(io_error)?.take(MAX_TEXT + 1).read_to_string(&mut s).map_err(io_error)?;
    if s.len() as u64 > MAX_TEXT { return Err(Failure::usage("Textdatei größer als 2 MiB.")); }
    Ok(s)
}
fn local_models() -> Result<Vec<String>> {
    let response = reqwest::blocking::Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).timeout(Duration::from_secs(3)).build().map_err(|e| Failure::error(e.to_string()))?.get("http://127.0.0.1:11434/api/tags").send().map_err(|_| Failure::error("Lokales Ollama nicht erreichbar."))?;
    if !response.status().is_success() { return Err(Failure::error("Ollama meldet einen Fehler.")); }
    let mut s = String::new();
    response.take(MAX_TEXT + 1).read_to_string(&mut s).map_err(io_error)?;
    if s.len() as u64 > MAX_TEXT { return Err(Failure::error("Ollama-Antwort zu groß.")); }
    ai::parse_ollama_tags(&s).map_err(Failure::error)
}
fn ai_action(cmd: &AiAction) -> Result<Value> {
    match cmd {
        AiAction::Models => Ok(json!({"models":local_models()?})),
        AiAction::Recommend { available_ram_mb } => {
            let (models, warning) = match local_models() { Ok(m) => (m, None), Err(e) => (vec![], Some(e.message)) };
            let mut v = ai::recommend_model(*available_ram_mb, &models); v["warning"] = json!(warning); Ok(v)
        }
        AiAction::Validate { response, sources } => {
            let raw = bounded_text(response)?;
            let v: Value = serde_json::from_str(&bounded_text(sources)?).map_err(|e| Failure::usage(e.to_string()))?;
            let items = v.as_array().ok_or_else(|| Failure::usage("Quellen müssen ein JSON-Array sein."))?;
            if items.len() > 16 { return Err(Failure::usage("Maximal 16 Quellen.")); }
            let mut trusted = Vec::new(); let mut ids = HashSet::new();
            for item in items {
                let id = item["id"].as_u64().and_then(|v| u32::try_from(v).ok()).ok_or_else(|| Failure::usage("Ungültige Quellen-ID."))?;
                if !ids.insert(id) { return Err(Failure::usage("Doppelte Quellen-ID.")); }
                let string = |key: &str| { item[key].as_str().map(str::to_owned).ok_or_else(|| Failure::usage(format!("Quelle ohne {key}."))) };
                let title = string("title")?; let url = string("url")?; let text = string("text")?;
                if title.chars().count() > 300 || url.chars().count() > 2000 || text.chars().count() > 30000 { return Err(Failure::usage("Quelle zu groß.")); }
                trusted.push(ai::TrustedSource { id, title, url, text });
            }
            let mut v = ai::validate_quotes(&raw, &trusted).map_err(Failure::error)?;
            v["source_origin"] = json!("user_provided_not_independently_fetched");
            v["provenance_verified"] = json!(false);
            v["warning"] = json!("Nur Textabgleich; keine unabhängige Quellen- oder Faktenprüfung.");
            Ok(v)
        }
    }
}
pub fn emit(v: &Value, json_mode: bool) -> Result<()> {
    let text = if json_mode { serde_json::to_string_pretty(v).map_err(|e| Failure::error(e.to_string()))? }
    else if let Some(items) = v.get("items").and_then(Value::as_array) {
        if items.is_empty() { "Keine Pakete gefunden.".into() } else { items.iter().map(|v| format!("{}  {}  {}", v["id"].as_str().unwrap_or(""), v["ver"].as_str().unwrap_or(""), v["nm"].as_str().unwrap_or(""))).collect::<Vec<_>>().join("\n") }
    } else if let Some(text) = v.get("text").and_then(Value::as_str) { text.into() }
    else if let Some(hash) = v.get("sha256").and_then(Value::as_str) { format!("{hash}  {}", v["path"].as_str().unwrap_or("")) }
    else if let Some(files) = v.get("files").and_then(Value::as_array) {
        let mut lines = files.iter().map(|v| format!("{}  {} Bytes  {}", v["kind"].as_str().unwrap_or(""), v["size"], v["path"].as_str().unwrap_or(""))).collect::<Vec<_>>();
        if lines.is_empty() { lines.push("Keine EXE/MSI-Dateien in diesem Ordner gefunden.".into()); }
        if let Some(w) = v.get("warnings").and_then(Value::as_array) { lines.extend(w.iter().filter_map(Value::as_str).map(|s| format!("Hinweis: {s}"))); }
        lines.join("\n")
    } else { serde_json::to_string_pretty(v).map_err(|e| Failure::error(e.to_string()))? };
    let text = if json_mode { text } else { terminal_text(&text) };
    writeln!(io::stdout().lock(), "{text}").map_err(|e| { if e.kind() == io::ErrorKind::BrokenPipe { Failure { code: 0, message: String::new() } } else { io_error(e) } })
}
pub fn menu(timeout: u64, cancel: Arc<AtomicBool>) -> Result<()> {
    if !io::stdin().is_terminal() { return Err(Failure::usage("Menü benötigt ein Terminal; für Skripte Unterbefehle verwenden.")); }
    let ctx = Context { cancel: cancel.clone(), timeout: Duration::from_secs(timeout) };
    let prompt = |label: &str| -> Result<Option<String>> { eprint!("{label} (leer = zurück): "); io::stderr().flush().map_err(io_error)?; Ok(read_input(&ctx)?.filter(|s| !s.is_empty())) };
    loop {
        ctx.check()?;
        eprintln!("\nAppmenu · CLI\n1 Info\n2 Pakete suchen\n3 Paketdetails\n4 Benutzerinstallation\n5 Dateien auflisten\n6 Dateiinformationen\n7 SHA-256\n8 EXE starten\n9 Lokale KI\n0 Beenden");
        let choice = match prompt("Auswahl")? { Some(s) => s, None => return Ok(()) };
        let action = match choice.as_str() {
            "0" | "q" => return Ok(()),
            "1" => Action::Info,
            "2" => match prompt("Suchbegriff")? { Some(query) => Action::Search { query }, None => continue },
            "3" => match prompt("Exakte Paket-ID")? { Some(id) => Action::Detail { id }, None => continue },
            "4" => match prompt("Exakte IDs, durch Leerzeichen getrennt")? { Some(s) => Action::Install(Install { ids: s.split_whitespace().map(str::to_owned).collect(), dry_run: false, yes: false }), None => continue },
            "5" => Action::Scan { dir: prompt("Ordner, leer = Downloads/Desktop")?.map(PathBuf::from), limit: 60 },
            "6" => match prompt("Dateipfad, ohne Anführungszeichen")? { Some(s) => Action::FileInfo { path: s.into() }, None => continue },
            "7" => match prompt("Dateipfad, ohne Anführungszeichen")? { Some(s) => Action::Hash { path: s.into() }, None => continue },
            "8" => match prompt("EXE-Pfad, ohne Anführungszeichen")? { Some(s) => Action::Open { path: s.into(), yes: false }, None => continue },
            "9" => {
                eprintln!("1 Installierte Ollama-Modelle\n2 Modellvorschlag\n3 Zitate prüfen");
                let command = match prompt("KI-Auswahl")?.as_deref() {
                    Some("1") => AiAction::Models,
                    Some("2") => {
                        let ram = match prompt("Freier RAM in MiB (1..1048576)")?.and_then(|s| s.parse::<u64>().ok()).filter(|v| (1..=1048576).contains(v)) { Some(v) => v, None => { eprintln!("Ungültiger RAM-Wert."); continue; } };
                        AiAction::Recommend { available_ram_mb: Some(ram) }
                    }
                    Some("3") => {
                        let response = match prompt("Antwort-JSON-Pfad")? { Some(s) => s.into(), None => continue };
                        let sources = match prompt("Quellen-JSON-Pfad")? { Some(s) => s.into(), None => continue };
                        AiAction::Validate { response, sources }
                    }
                    _ => continue,
                };
                Action::Ai { command }
            }
            _ => { eprintln!("Bitte eine angezeigte Nummer wählen."); continue; }
        };
        match execute(&Cli { json: false, timeout, command: Some(action) }, cancel.clone()).and_then(|v| emit(&v, false)) {
            Ok(()) => {}
            Err(e) if e.code == 130 => return Err(e),
            Err(e) => eprintln!("Fehler: {}", terminal_text(&e.message)),
        }
    }
}
