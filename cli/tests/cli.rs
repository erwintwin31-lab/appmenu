use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_appmenu-cli"))
}

fn temporary_directory() -> std::path::PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "appmenu-cli-test-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&path).unwrap();
    path
}

#[test]
fn help_and_version_work() {
    assert!(cli().arg("--help").output().unwrap().status.success());
    assert!(cli().arg("--version").output().unwrap().status.success());
}

#[test]
fn info_is_a_json_document() {
    let result = cli().args(["info", "--json"]).output().unwrap();
    assert!(result.status.success());
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(value["capabilities"].is_object());
    assert!(result.stderr.is_empty());
}

#[test]
fn unknown_command_returns_structured_error() {
    let result = cli().args(["--json", "invalid-command"]).output().unwrap();
    assert_eq!(result.status.code(), Some(2));
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["ok"], false);
}

#[test]
fn menu_rejects_nonterminal_and_json_mode() {
    for arguments in [vec!["menu"], vec!["menu", "--json"]] {
        let result = cli()
            .args(arguments)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2));
    }
}

#[test]
fn stdin_hash_matches_known_sha256() {
    let mut child = cli()
        .args(["hash", "-", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"abc").unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success());
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(
        value["sha256"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn file_hash_handles_spaces_and_missing_files() {
    let directory = temporary_directory();
    let file = directory.join("a file with spaces.txt");
    std::fs::write(&file, b"abc").unwrap();
    let result = cli().arg("hash").arg(&file).arg("--json").output().unwrap();
    assert!(result.status.success());
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(
        value["sha256"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let missing = cli()
        .arg("hash")
        .arg(directory.join("missing"))
        .arg("--json")
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing.stdout).unwrap()["ok"],
        false
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn option_like_package_ids_are_rejected() {
    let result = cli()
        .args(["install", "--json", "--", "--scope"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(serde_json::from_slice::<Value>(&result.stdout).is_ok());
}
