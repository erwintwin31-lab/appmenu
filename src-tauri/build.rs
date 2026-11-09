use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=tauri.linux.conf.json");
    println!("cargo:rerun-if-changed=tauri.windows.conf.json");
    println!("cargo:rerun-if-changed=../cli/Cargo.toml");
    println!("cargo:rerun-if-changed=../cli/Cargo.lock");
    println!("cargo:rerun-if-changed=../cli/build.rs");
    println!("cargo:rerun-if-changed=../cli/appmenu-cli.manifest");
    println!("cargo:rerun-if-changed=../cli/src");
    println!("cargo:rerun-if-changed=src/ai.rs");
    println!("cargo:rerun-if-changed=src/safety.rs");
    println!("cargo:rerun-if-changed=src/packages.rs");
    let platform = env::var("CARGO_CFG_TARGET_OS").expect("target OS");
    if matches!(platform.as_str(), "linux" | "windows") {
        let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest"));
        let root = manifest.parent().expect("repository root");
        let target = env::var("TARGET").expect("target triple");
        let target_dir = root.join("cli/target");
        let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let status = Command::new(cargo)
            .current_dir(root)
            .args([
                "build",
                "--locked",
                "--release",
                "--jobs",
                "2",
                "--manifest-path",
                "cli/Cargo.toml",
                "--target",
                target.as_str(),
            ])
            .arg("--target-dir")
            .arg(&target_dir)
            .env_remove("CARGO_MAKEFLAGS")
            .env_remove("MAKEFLAGS")
            .env_remove("MFLAGS")
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .status()
            .expect("start standalone CLI build");
        assert!(status.success(), "standalone CLI build failed");
        let name = if platform == "windows" {
            "appmenu-cli.exe"
        } else {
            "appmenu-cli"
        };
        let source = target_dir.join(&target).join("release").join(name);
        let dist = root.join("cli/dist");
        fs::create_dir_all(&dist).expect("create CLI resource directory");
        let destination = dist.join(name);
        fs::copy(source, &destination).expect("copy CLI resource");
        #[cfg(unix)]
        if platform == "linux" {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(destination, fs::Permissions::from_mode(0o755))
                .expect("set executable permissions");
        }
    }
    tauri_build::build()
}
