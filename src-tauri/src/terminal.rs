//! Fixed native launcher for the bundled CLI. No frontend command/path arguments.
use serde_json::{json, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tauri::Manager;

#[derive(Clone, Copy)]
enum Kind {
    Gnome,
    Exec,
}

fn cli_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let path = app
        .path()
        .resolve("appmenu-cli", tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    let path = std::fs::canonicalize(path)
        .map_err(|_| "Die mitgelieferte CLI wurde nicht gefunden.".to_string())?;
    if !path.is_file() {
        return Err("Die CLI-Ressource ist keine reguläre Datei.".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if path.metadata().map_err(|e| e.to_string())?.permissions().mode() & 0o111 == 0 {
            return Err("Die CLI-Ressource ist nicht ausführbar.".into());
        }
    }
    Ok(path)
}

fn terminal() -> Result<(PathBuf, Kind), String> {
    if !cfg!(target_os = "linux") {
        return Err("GUI-Terminalstart ist derzeit nur für Linux verfügbar.".into());
    }
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn geteuid() -> u32;
        }
        // SAFETY: Read-only query without arguments or pointer access.
        if unsafe { geteuid() } == 0 {
            return Err("Appmenu bitte ohne root-Rechte starten.".into());
        }
    }
    let graphical = ["DISPLAY", "WAYLAND_DISPLAY"]
        .iter()
        .any(|key| std::env::var_os(key).is_some_and(|value| !value.is_empty()));
    if !graphical {
        return Err("Keine grafische Sitzung erkannt. Die CLI separat starten.".into());
    }
    for (name, kind) in [
        ("/usr/bin/x-terminal-emulator", Kind::Exec),
        ("/usr/bin/gnome-terminal", Kind::Gnome),
        ("/usr/bin/konsole", Kind::Exec),
        ("/usr/bin/xterm", Kind::Exec),
    ] {
        let path = PathBuf::from(name);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if path
                .metadata()
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            {
                return Ok((path, kind));
            }
        }
        #[cfg(not(unix))]
        let _ = (path, kind);
    }
    Err("Kein unterstützter Terminalstarter gefunden.".into())
}

fn args(kind: Kind, cli: &Path) -> Vec<OsString> {
    let separator = match kind {
        Kind::Gnome => "--",
        Kind::Exec => "-e",
    };
    vec![separator.into(), cli.as_os_str().to_owned(), "menu".into()]
}

pub(crate) fn status(app: &tauri::AppHandle) -> Value {
    let result = terminal().and_then(|_| cli_path(app).map(|_| ()));
    match result {
        Ok(()) => json!({
            "available": true,
            "platform": std::env::consts::OS,
            "reason": "Appmenu CLI in einem separaten Terminal öffnen."
        }),
        Err(reason) => json!({
            "available": false,
            "platform": std::env::consts::OS,
            "reason": reason
        }),
    }
}

pub(crate) fn launch(app: &tauri::AppHandle) -> Result<Value, String> {
    let (program, kind) = terminal()?;
    let cli = cli_path(app)?;
    let mut child = Command::new(program)
        .args(args(kind, &cli))
        .current_dir(cli.parent().ok_or("CLI-Ordner fehlt.")?)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Terminalstart fehlgeschlagen: {e}"))?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(json!({
        "started": true,
        "message": "Terminalstarter gestartet; die Oberfläche bleibt geöffnet."
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_preserve_literal_path() {
        let path = Path::new("/opt/Appmenu Test;literal/appmenu-cli");
        let values = args(Kind::Exec, path);
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], "-e");
        assert_eq!(values[1], path.as_os_str());
        assert_eq!(values[2], "menu");
    }

    #[test]
    fn gnome_uses_argument_separator() {
        let values = args(Kind::Gnome, Path::new("/opt/appmenu-cli"));
        assert_eq!(values[0], "--");
        assert_eq!(values[2], "menu");
    }
}
