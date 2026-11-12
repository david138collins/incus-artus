//! The one doorway to Bevy [the regroup, David 2026-11-05: "we're not having any mercy on bevy...
//! if we keep anything in place in the meantime - we need to build explosive fastener ejection seats
//! for it"].
//!
//! Bevy is kept only for its core (the World, its resources and messages, the schedules, states,
//! time, input types and logging), and only because replacing it now isn't worth the risk, not
//! because it's best. Everything Bevy is reached through here (`crate::ecs::prelude::*` and so on),
//! never `bevy::` directly, so pulling it out means giving these names to our own code in one place.
//!
//! **The gauge** (the test below): no file but this one names `bevy::`, and the number of files and
//! lines that use the doorway may only go down. New code doesn't use it at all unless it has to talk
//! to the World (docs/regroup.md, "The ejection seat").

pub use bevy::*;

/// The doorway's reach when the gauge was last moved: files, and lines, that use `crate::ecs`. Lower
/// them as Bevy is pulled out; never raise them.
#[cfg(test)]
const GAUGE: (usize, usize) = (51, 83);

#[cfg(test)]
mod tests {
    use super::GAUGE;

    /// Every `.rs` file under `src/`, with its text.
    fn sources() -> Vec<(std::path::PathBuf, String)> {
        fn walk(folder: &std::path::Path, found: &mut Vec<(std::path::PathBuf, String)>) {
            for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, found);
                } else if path.extension().is_some_and(|e| e == "rs")
                    && let Ok(text) = std::fs::read_to_string(&path)
                {
                    found.push((path, text));
                }
            }
        }
        let mut found = Vec::new();
        walk(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut found);
        found
    }

    #[test]
    fn bevy_is_reached_only_through_the_doorway_and_its_reach_only_shrinks() {
        let mut direct = Vec::new();
        let (mut files, mut lines) = (0, 0);
        for (path, text) in sources() {
            if path.ends_with("ecs.rs") {
                continue;
            }
            let code: Vec<&str> = text.lines().filter(|line| !line.trim_start().starts_with("//")).collect();
            direct.extend(code.iter().filter(|line| line.contains("bevy::")).map(|line| format!("{}: {}", path.display(), line.trim())));
            let using = code.iter().filter(|line| line.contains("crate::ecs")).count();
            if using > 0 {
                files += 1;
                lines += using;
            }
        }
        assert!(direct.is_empty(), "Bevy named outside the doorway (use crate::ecs):\n{}", direct.join("\n"));
        assert!(files <= GAUGE.0 && lines <= GAUGE.1, "the doorway's reach grew: {files} files, {lines} lines (the gauge: {} files, {} lines); it may only go down", GAUGE.0, GAUGE.1);
        if (files, lines) != GAUGE {
            println!("The doorway's reach is now {files} files and {lines} lines: lower GAUGE in src/ecs.rs to that.");
        }
    }
}
