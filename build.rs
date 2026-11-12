//! Build script: stamps the game with the commit it was built from, for the session's log and the flights' reports.
//! "+ changes" means the code (src, Cargo.toml, Cargo.lock, build.rs) differed from the commit, and is followed by a
//! short fingerprint of the difference, so two builds with different changes never share a stamp. The changed files
//! are named (`INCUS_BUILD_CHANGED`) and the difference itself is kept beside the build (`changes.patch` in its own
//! folder), so a flight can say exactly what flew. Without git, the stamp is "unknown". It also reads the build's
//! number from the `BUILD` file, if there is one. [Carried from old's, which learnt it: the Oracle's experiment 1,
//! 2026-11-11.]

use std::process::Command;

/// What a build is made from (the journal, the art and the notes are outside it, and other people edit them).
const CODE: [&str; 4] = ["src", "Cargo.toml", "Cargo.lock", "build.rs"];

fn main() {
    let git = |args: &[&str]| Command::new("git").args(args).output().ok().filter(|out| out.status.success()).map(|out| String::from_utf8_lossy(&out.stdout).into_owned());
    fn with_code<'a>(args: &[&'a str]) -> Vec<&'a str> {
        args.iter().copied().chain(["--"]).chain(CODE).collect()
    }
    let commit = git(&["rev-parse", "--short", "HEAD"]).map(|out| out.trim().to_string());
    // Where this folder is in the repository: git's status names files from the repository's top, so they're named
    // from here by taking it off (nothing, while the project is at the top).
    let prefix = git(&["rev-parse", "--show-prefix"]).map(|out| out.trim().to_string()).unwrap_or_default();
    // What differs from the commit: changed files, and new ones git doesn't know yet.
    let status = git(&with_code(&["status", "--porcelain", "--untracked-files=all"])).unwrap_or_default();
    let files: Vec<(bool, String)> = status
        .lines()
        .filter(|line| line.len() > 3)
        .map(|line| {
            let path = line[3..].trim_matches('"');
            (line.starts_with("??"), path.strip_prefix(prefix.as_str()).unwrap_or(path).to_string())
        })
        .collect();
    // The difference itself: the changed files' diff, and the new code files whole.
    let mut patch = git(&with_code(&["diff", "HEAD"])).unwrap_or_default();
    for (_, path) in files.iter().filter(|(new, path)| *new && path.starts_with("src/")) {
        // `git diff --no-index` exits 1 when the files differ, which they always do here.
        if let Ok(out) = Command::new("git").args(["diff", "--no-index", "--", "/dev/null", path]).output() {
            patch += &String::from_utf8_lossy(&out.stdout);
        }
    }
    let stamp = match commit {
        Some(commit) if !files.is_empty() => format!("{commit} + changes {:08x}", fnv(&patch) as u32),
        Some(commit) => commit,
        None => "unknown".to_string(),
    };
    println!("cargo:rustc-env=INCUS_BUILD={stamp}");
    // The changed files by name (new ones marked), for the report's header.
    let mut named: Vec<String> = files.iter().map(|(new, path)| if *new { format!("{path} (new)") } else { path.clone() }).collect();
    if named.len() > 24 {
        let more = named.len() - 24;
        named.truncate(24);
        named.push(format!("and {more} more"));
    }
    println!("cargo:rustc-env=INCUS_BUILD_CHANGED={}", named.join(", "));
    let out = std::env::var("OUT_DIR").unwrap_or_else(|_| ".".into());
    let _ = std::fs::write(std::path::Path::new(&out).join("changes.patch"), patch);
    // The build's number (the `BUILD` file: one more for each build handed to David to play).
    let number = std::fs::read_to_string("BUILD").map(|text| text.trim().to_string()).unwrap_or_default();
    println!("cargo:rustc-env=INCUS_BUILD_NUMBER={number}");
    println!("cargo:rerun-if-changed=BUILD");
    // Stamped again when the commit or the code changes: the repository's own folder, wherever it is from here (a
    // watched file that doesn't exist would build the game afresh every time).
    let git_dir = git(&["rev-parse", "--git-dir"]).map(|out| out.trim().to_string()).unwrap_or_else(|| ".git".to_string());
    println!("cargo:rerun-if-changed={git_dir}/HEAD");
    println!("cargo:rerun-if-changed={git_dir}/refs/heads");
    for path in CODE {
        println!("cargo:rerun-if-changed={path}");
    }
}

/// A short fingerprint of the difference (64-bit FNV-1a).
fn fnv(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| (hash ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3))
}
