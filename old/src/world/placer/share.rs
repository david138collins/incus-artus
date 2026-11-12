//! Sharing (stop 4 of `docs/placer.md`): a distributor or a made field exported to a file of its
//! own, and imported into any preset, for sharing and mods.
//!
//! - **A shared file** holds what was exported and the made fields it reads (and those they read),
//!   in order, so it works wherever it's imported. A distributor's "chosen by" another distributor
//!   isn't carried: what's before it depends on where it's imported, and a missing one shows as a
//!   problem on its layer.
//! - **Importing** puts it at the end of its list. A made field it needs that the preset hasn't got
//!   comes too; one of the same name the preset has already is kept (the file's is left out, and
//!   it says so). A distributor or a field whose own name is taken is renamed.
//! - **The format is the presets' own** (the same TOML), so a file can be read and edited by hand,
//!   and moving to mods' data files later is cheap (`docs/placer.md` section 6).

use super::{Distributor, MadeField, Source};
use crate::world::MineralogySettings;
use serde::{Deserialize, Serialize};

/// What a shared file holds: the made fields needed, in order (an exported field last), and the
/// distributor exported, if it was one.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shared {
    pub fields: Vec<MadeField>,
    pub distributors: Vec<Distributor>,
}

/// The made fields the layers read (by name), and every one those read, in the preset's order.
fn needed<'a>(m: &MineralogySettings, layers: impl IntoIterator<Item = &'a super::Layer>) -> Vec<MadeField> {
    let mut wanted: Vec<String> = Vec::new();
    let mut to_visit: Vec<String> = layers.into_iter().filter_map(|layer| if let Source::Made { name } = &layer.source { Some(name.clone()) } else { None }).collect();
    while let Some(name) = to_visit.pop() {
        if wanted.contains(&name) {
            continue;
        }
        if let Some(made) = m.fields.iter().find(|made| made.name == name) {
            to_visit.extend(made.layers.iter().filter_map(|layer| if let Source::Made { name } = &layer.source { Some(name.clone()) } else { None }));
            wanted.push(name);
        }
    }
    m.fields.iter().filter(|made| wanted.contains(&made.name)).cloned().collect()
}

/// Made field `k`, and the made fields it reads.
pub fn export_field(m: &MineralogySettings, k: usize) -> Option<Shared> {
    let made = m.fields.get(k)?;
    let mut fields = needed(m, &made.layers);
    fields.push(made.clone());
    Some(Shared { fields, distributors: Vec::new() })
}

/// Distributor `d`, and the made fields its candidates read.
pub fn export_distributor(m: &MineralogySettings, d: usize) -> Option<Shared> {
    let distributor = m.distributors.get(d)?;
    let fields = needed(m, distributor.candidates.iter().flat_map(|candidate| &candidate.layers));
    Some(Shared { fields, distributors: vec![distributor.clone()] })
}

/// `name`, or the first of "`name` 2", "`name` 3"… that `taken` says is free.
fn free_name(name: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(name) {
        return name.to_string();
    }
    (2..).map(|n| format!("{name} {n}")).find(|wanted| !taken(wanted)).unwrap_or_default()
}

/// Puts what a shared file holds into the preset, at the end of its lists; says what it did.
pub fn import(m: &mut MineralogySettings, shared: Shared) -> Vec<String> {
    let mut notes = Vec::new();
    let last_field = shared.fields.len().saturating_sub(1);
    let exported_a_field = shared.distributors.is_empty();
    for (k, made) in shared.fields.into_iter().enumerate() {
        let is_the_one = exported_a_field && k == last_field;
        match m.fields.iter().find(|own| own.name == made.name) {
            // A field it reads, the preset has its own: kept.
            Some(own) if !is_the_one => {
                if *own != made {
                    notes.push(format!("kept your own field \"{}\" (the file's differs)", made.name));
                }
            }
            // The one exported: in, renamed if its name is taken.
            _ => {
                let name = free_name(&made.name, |name| m.fields.iter().any(|own| own.name == name));
                if name != made.name {
                    notes.push(format!("the field \"{}\" came in as \"{name}\"", made.name));
                } else {
                    notes.push(format!("added the field \"{name}\""));
                }
                m.fields.push(MadeField { name, ..made });
            }
        }
    }
    for distributor in shared.distributors {
        let name = free_name(&distributor.name, |name| m.distributors.iter().any(|own| own.name == name));
        notes.push(if name != distributor.name { format!("the distributor \"{}\" came in as \"{name}\", at the end", distributor.name) } else { format!("added the distributor \"{name}\", at the end") });
        m.distributors.push(Distributor { name, ..distributor });
    }
    notes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::placer::{Candidate, Layer};

    fn reads(name: &str) -> Layer {
        Layer { source: Source::Made { name: name.into() }, ..Layer::default() }
    }

    fn field(name: &str, layers: Vec<Layer>) -> MadeField {
        MadeField { name: name.into(), layers }
    }

    /// Fields "wet", "low" (reads nothing), "wet and low" (reads both), "other"; a distributor whose
    /// candidate reads "wet and low".
    fn preset() -> MineralogySettings {
        let fields = vec![field("wet", vec![Layer::default()]), field("low", vec![Layer::default()]), field("wet and low", vec![reads("wet"), reads("low")]), field("other", vec![])];
        let candidate = Candidate { block: "incus:sand".into(), layers: vec![reads("wet and low")], ..Candidate::default() };
        MineralogySettings { fields, distributors: vec![Distributor { name: "beaches".into(), candidates: vec![candidate], ..Distributor::default() }], ..MineralogySettings::default() }
    }

    #[test]
    fn an_export_carries_the_fields_it_reads_and_survives_its_file() {
        let m = preset();
        let shared = export_distributor(&m, 0).unwrap();
        let names: Vec<&str> = shared.fields.iter().map(|made| made.name.as_str()).collect();
        assert_eq!(names, ["wet", "low", "wet and low"], "what it reads, and what that reads, in order; not \"other\"");
        let back: Shared = toml::from_str(&toml::to_string_pretty(&shared).unwrap()).unwrap();
        assert_eq!(back, shared);
        let one = export_field(&m, 2).unwrap();
        assert_eq!(one.fields.last().map(|made| made.name.as_str()), Some("wet and low"));
        assert!(one.distributors.is_empty());
    }

    #[test]
    fn importing_adds_at_the_end_keeps_your_own_fields_and_renames_what_s_taken() {
        let m = preset();
        let shared = export_distributor(&m, 0).unwrap();
        // Into an empty preset: everything comes.
        let mut empty = MineralogySettings { fields: Vec::new(), distributors: Vec::new(), ..MineralogySettings::default() };
        import(&mut empty, shared.clone());
        assert_eq!((empty.fields.len(), empty.distributors[0].name.as_str()), (3, "beaches"));
        // Into the preset it came from: its fields are kept, the distributor renamed.
        let mut again = preset();
        let notes = import(&mut again, shared);
        assert_eq!((again.fields.len(), again.distributors.len(), again.distributors[1].name.as_str()), (4, 2, "beaches 2"));
        assert!(notes.iter().any(|note| note.contains("came in as")));
        // A field of the same name that differs: the preset's is kept, and it says so.
        let mut changed = preset();
        changed.fields[0].layers.clear();
        let notes = import(&mut changed, export_distributor(&preset(), 0).unwrap());
        assert!(changed.fields[0].layers.is_empty() && notes.iter().any(|note| note.contains("kept your own field \"wet\"")), "{notes:?}");
        // A made field exported and brought back: in again, renamed.
        let mut fields = preset();
        import(&mut fields, export_field(&preset(), 1).unwrap());
        assert_eq!(fields.fields.last().map(|made| made.name.as_str()), Some("low 2"));
    }
}
