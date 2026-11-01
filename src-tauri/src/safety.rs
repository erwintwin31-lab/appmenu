//! Fail-closed checks shared by capability reporting and native execution.
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn execution_allowed(elevation: &str) -> bool {
    matches!(elevation, "standard" | "admin_not_elevated")
}

pub fn require_normal_process() -> Result<(), String> {
    if execution_allowed(elevation()) {
        Ok(())
    } else {
        Err("Appmenu muss mit ermittelbaren, nicht erhöhten Benutzerrechten laufen. Bitte normal neu starten.".into())
    }
}

pub fn winget_path() -> Result<PathBuf, String> {
    if !cfg!(windows) {
        return Err("winget ist nur unter Windows verfügbar.".into());
    }
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or_else(|| "Der Benutzerordner konnte nicht ermittelt werden.".to_string())?;
    let path = base
        .join("Microsoft")
        .join("WindowsApps")
        .join("winget.exe");
    if !path.is_file() {
        return Err("Der winget-Ausführungsalias wurde nicht gefunden.".into());
    }
    Ok(path)
}

pub struct InstallGuard {
    flag: std::sync::Arc<AtomicBool>,
}

impl InstallGuard {
    pub fn acquire(flag: std::sync::Arc<AtomicBool>) -> Result<Self, String> {
        flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "Es läuft bereits eine Installation.".to_string())?;
        Ok(Self { flag })
    }
}

impl Drop for InstallGuard {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::Release);
    }
}

#[cfg(not(windows))]
pub fn elevation() -> &'static str {
    "unknown"
}

#[cfg(windows)]
pub fn elevation() -> &'static str {
    use std::ffi::c_void;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }
    #[link(name = "advapi32")]
    extern "system" {
        fn OpenProcessToken(process: *mut c_void, access: u32, token: *mut *mut c_void) -> i32;
        fn GetTokenInformation(
            token: *mut c_void,
            class: u32,
            buffer: *mut c_void,
            size: u32,
            returned: *mut u32,
        ) -> i32;
    }

    struct Token(*mut c_void);
    impl Drop for Token {
        fn drop(&mut self) {
            // SAFETY: The handle was returned by OpenProcessToken and is closed once.
            unsafe { CloseHandle(self.0) };
        }
    }

    // SAFETY: Read-only queries on our own process. Both queried structures are one DWORD.
    unsafe {
        let mut raw = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), 0x0008, &mut raw) == 0 {
            return "unknown";
        }
        let token = Token(raw);
        let mut elevated = 0u32;
        let mut kind = 0u32;
        let mut returned = 0u32;
        if GetTokenInformation(
            token.0,
            20, // TokenElevation
            (&mut elevated as *mut u32).cast(),
            4,
            &mut returned,
        ) == 0
            || returned != 4
        {
            return "unknown";
        }
        if GetTokenInformation(
            token.0,
            18, // TokenElevationType
            (&mut kind as *mut u32).cast(),
            4,
            &mut returned,
        ) == 0
            || returned != 4
        {
            return "unknown";
        }
        if elevated != 0 {
            "elevated"
        } else if kind == 3 {
            "admin_not_elevated"
        } else {
            "standard"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn actions_require_known_non_elevated_rights() {
        assert!(execution_allowed("standard"));
        assert!(execution_allowed("admin_not_elevated"));
        for mode in ["elevated", "unknown", "", "unexpected"] {
            assert!(!execution_allowed(mode));
        }
    }

    #[test]
    fn install_lock_is_exclusive_and_released_on_drop() {
        let flag = Arc::new(AtomicBool::new(false));
        let first = InstallGuard::acquire(flag.clone()).unwrap();
        assert!(InstallGuard::acquire(flag.clone()).is_err());
        drop(first);
        assert!(InstallGuard::acquire(flag).is_ok());
    }

    #[test]
    fn install_lock_is_released_on_early_error() {
        fn fail(flag: Arc<AtomicBool>) -> Result<(), String> {
            let _guard = InstallGuard::acquire(flag)?;
            Err("preflight failed".into())
        }
        let flag = Arc::new(AtomicBool::new(false));
        assert!(fail(flag.clone()).is_err());
        assert!(InstallGuard::acquire(flag).is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn own_token_is_readable_without_whoami() {
        assert!(matches!(
            elevation(),
            "standard" | "admin_not_elevated" | "elevated"
        ));
    }

    #[cfg(not(windows))]
    #[test]
    fn other_platforms_do_not_impersonate_windows_users() {
        assert_eq!(elevation(), "unknown");
        assert!(winget_path().is_err());
    }
}
