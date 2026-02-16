use std::process::Command;

fn main() {
    set_git_hash();
    set_build_time();
    set_target();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");
}

fn set_git_hash() {
    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_HASH={hash}");
}

fn set_build_time() {
    let time = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    println!("cargo:rustc-env=BUILD_TIME={time}");
}

fn set_target() {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=TARGET={target}");
}
