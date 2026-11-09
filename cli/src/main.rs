use appmenu_cli::{emit, execute, menu, terminal_text, Action, Cli, Failure};
use clap::Parser;
use serde_json::json;
use std::{
    io::{self, IsTerminal},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args_os().collect();
    let wants_json = args
        .iter()
        .take_while(|s| *s != "--")
        .any(|s| s == "--json");
    let cli = match Cli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            let code = e.exit_code() as u8;
            if code != 0 && wants_json {
                let _ = emit(
                    &json!({"ok":false,"error":{"code":code,"message":e.to_string()}}),
                    true,
                );
            } else {
                let _ = e.print();
            }
            return code.into();
        }
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    let result = ctrlc::set_handler(move || {
        flag.store(true, Ordering::Release);
    })
    .map_err(|e| Failure::error(format!("Abbruchhandler: {e}")))
    .and_then(|_| {
        let is_menu = matches!(cli.command, Some(Action::Menu))
            || (cli.command.is_none() && io::stdin().is_terminal() && !cli.json);
        if is_menu {
            if cli.json {
                Err(Failure::usage("menu und --json sind nicht kombinierbar."))
            } else {
                menu(cli.timeout, cancel)
            }
        } else {
            execute(&cli, cancel).and_then(|v| emit(&v, cli.json))
        }
    });
    match result {
        Ok(()) => 0.into(),
        Err(e) if e.code == 0 => 0.into(),
        Err(e) => {
            if cli.json {
                let _ = emit(
                    &json!({"ok":false,"error":{"code":e.code,"message":e.message}}),
                    true,
                );
            } else {
                eprintln!("Fehler: {}", terminal_text(&e.message));
            }
            e.code.into()
        }
    }
}
