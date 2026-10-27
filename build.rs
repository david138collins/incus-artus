//! Build script: stamps the game with the commit it was built from, for crash and freeze reports
//! (see `src/diagnostics.rs`). "+ changes" means the code (src, assets, Cargo.toml) had changes not
//! yet committed. Without git, the stamp is "unknown".

use std::process::Command;

fn main() {
    let git = |args: &[&str]| Command::new("git").args(args).output().ok().filter(|out| out.status.success());
    let commit = git(&["rev-parse", "--short", "HEAD"]).map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string());
    // Uncommitted changes to the code (not docs or art, which other people edit too).
    let changed = Command::new("git").args(["diff", "--quiet", "HEAD", "--", "src", "assets", "Cargo.toml", "build.rs"]).status().map(|status| !status.success()).unwrap_or(false);
    let stamp = match commit {
        Some(commit) if changed => format!("{commit} + changes"),
        Some(commit) => commit,
        None => "unknown".to_string(),
    };
    println!("cargo:rustc-env=INCUS_BUILD={stamp}");
    // Stamped again when the commit or the code changes.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=Cargo.toml");
}
