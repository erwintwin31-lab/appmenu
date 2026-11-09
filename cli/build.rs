use std::{env, path::PathBuf};
fn main() {
    println!("cargo:rerun-if-changed=appmenu-cli.manifest");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        let manifest =
            PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("appmenu-cli.manifest");
        println!("cargo:rustc-link-arg-bin=appmenu-cli=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bin=appmenu-cli=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
}
