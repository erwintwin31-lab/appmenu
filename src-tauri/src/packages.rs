//! Shell-free WinGet helpers shared by native GUI and CLI.
use crate::safety;
use serde_json::{json, Value};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};

pub(crate) fn winget_command() -> Result<Command, String> {
    let mut c = Command::new(safety::winget_path()?);
    c.stdin(Stdio::null());
    #[cfg(windows)]
    c.creation_flags(0x0800_0000);
    Ok(c)
}

#[allow(dead_code)] // GUI helper; CLI adds bounded capture and cancellation.
pub(crate) fn winget_failure(action: &str, output: &std::process::Output) -> String {
    let code = output.status.code().unwrap_or(-1);
    let details: String = String::from_utf8_lossy(&output.stderr)
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .take(1200)
        .collect();
    format!(
        "{action} fehlgeschlagen (winget-Code {code}). {}",
        details.trim()
    )
}

pub(crate) fn user_installer_args<'a>(action: &'a str, id: &'a str) -> Vec<&'a str> {
    let mut args = vec![
        action,
        "--id",
        id,
        "--exact",
        "--scope",
        "user",
        "--source",
        "winget",
        "--accept-source-agreements",
        "--disable-interactivity",
    ];
    if action == "install" {
        args.push("--accept-package-agreements");
    }
    args
}

#[allow(dead_code)] // GUI helper; CLI adds bounded capture and cancellation.
pub(crate) fn preflight_user_installer(id: &str) -> Result<String, String> {
    if !valid_id(id) {
        return Err("Ungültige Paket-ID.".into());
    }
    let out = winget_command()?
        .args(user_installer_args("show", id))
        .output()
        .map_err(|e| format!("Paketprüfung konnte nicht gestartet werden: {e}"))?;
    if !out.status.success() {
        return Err(winget_failure("Prüfung der Benutzerinstallation", &out));
    }
    let lines: Vec<String> = clean_lines(&String::from_utf8_lossy(&out.stdout))
        .into_iter()
        .take(45)
        .collect();
    if lines.is_empty() {
        return Err("winget lieferte keine lesbaren Paketdetails.".into());
    }
    Ok(lines.join("\n"))
}

pub(crate) fn valid_id(s: &str) -> bool {
    let first_ok = s.starts_with(|c: char| c.is_ascii_alphanumeric());
    first_ok
        && s.len() <= 100
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
}

pub(crate) fn clean_lines(text: &str) -> Vec<String> {
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

pub(crate) fn find_word(h: &[char], word: &str) -> Option<usize> {
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

pub(crate) fn first_token(c: &[char]) -> String {
    let s: String = c.iter().collect();
    s.split_whitespace().next().unwrap_or("").to_string()
}

pub(crate) fn parse_table(lines: &[String]) -> Vec<Value> {
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

#[allow(dead_code)] // GUI helper; CLI adds bounded capture and cancellation.
pub(crate) fn winget_version() -> Option<String> {
    winget_command()
        .ok()?
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
