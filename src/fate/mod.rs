//! Fate — the Goddess of Fate: one probability engine for every random roll in the living world.
//!
//! Every roll says three things:
//! - its **category**, a dotted name like `world.raid` or `combat.hit`, so Fate knows what kind of
//!   thing is being decided;
//! - its **base chance** (0 to 1);
//! - whether **the player wants it to happen** (`Wanted::Yes`, `No`, or `Either`).
//!
//! Fate holds **twists**: each bends the odds of the categories it matches. A twist has two dials:
//! - **luck**: towards whatever the player wants (negative = against the player);
//! - **strangeness**: towards things happening at all, whatever the player wants.
//!
//! Dials are in "doublings of the odds": +1 doubles the odds, −1 halves them, +3.3 turns a
//! 1-in-100 chance into about 1-in-10. Odds can never pass "certain", and a chance of exactly 0
//! or 1 is never bent (impossible stays impossible).
//!
//! World generation doesn't go through Fate: it uses position hashes (see `dice.rs`), so the same
//! seed always makes the same planet. Players are never told any of this exists.

mod dice;

#[allow(unused_imports)] // planet_hash is first used by the world-gen suite
pub use dice::{Dice, derive_seed, planet_hash};

use crate::debug::DebugAppExt;
use crate::save::{WorldFolder, WorldStateAppExt};
use crate::time::{WorldClock, describe_moment};
use bevy::prelude::*;
use bevy_egui::egui;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

/// Does the player want this to happen?
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Wanted {
    Yes,
    No,
    Either,
}

impl Wanted {
    /// Which way luck pushes: +1 towards "yes", −1 towards "no", 0 not at all.
    fn sign(self) -> f64 {
        match self {
            Wanted::Yes => 1.0,
            Wanted::No => -1.0,
            Wanted::Either => 0.0,
        }
    }
}

/// One bend in the odds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Twist {
    pub id: u32,
    /// Why it exists, for the debug page ("ignored the Goddess's call").
    pub label: String,
    /// Which categories it touches: `world.*` (that family), `loot.rare` (just that one), or `*`.
    pub categories: String,
    /// Doublings of the odds towards what the player wants (negative = against).
    pub luck: f64,
    /// Doublings of the odds towards things happening at all.
    pub strangeness: f64,
    /// World clock second it ends at (`None` = until removed).
    pub ends_at: Option<f64>,
}

impl Default for Twist {
    fn default() -> Self {
        Self {
            id: 0,
            label: "test twist".to_string(),
            categories: "*".to_string(),
            luck: 0.0,
            strangeness: 0.0,
            ends_at: None,
        }
    }
}

impl Twist {
    pub fn matches(&self, category: &str) -> bool {
        category_matches(&self.categories, category)
    }
}

/// `*` matches everything; `world.*` matches `world` and anything starting `world.`;
/// anything else must match exactly. Several patterns can be given separated by commas.
pub fn category_matches(patterns: &str, category: &str) -> bool {
    patterns.split(',').map(str::trim).any(|pattern| {
        if pattern == "*" {
            true
        } else if let Some(family) = pattern.strip_suffix(".*") {
            category == family || category.strip_prefix(family).is_some_and(|rest| rest.starts_with('.'))
        } else {
            pattern == category
        }
    })
}

/// Bend a chance by some doublings of the odds.
pub fn bend_chance(chance: f64, doublings: f64) -> f64 {
    if chance <= 0.0 || chance >= 1.0 || doublings == 0.0 {
        return chance.clamp(0.0, 1.0);
    }
    let odds = chance / (1.0 - chance) * doublings.exp2();
    odds / (1.0 + odds)
}

/// A d20 roll for the bookkeeping: what was needed, the face shown, and whether it succeeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct D20 {
    pub needed: u8,
    pub rolled: u8,
    pub success: bool,
}

/// One remembered roll, for the debug page.
#[derive(Clone, Debug)]
pub struct RollRecord {
    pub category: String,
    pub base: f64,
    pub bent: f64,
    pub result: bool,
    pub at: f64,
}

/// Running totals per category, for the debug page.
#[derive(Clone, Debug, Default)]
pub struct CategoryStats {
    pub rolls: u64,
    pub successes: u64,
    /// Sum of the base chances (so the average says what would have happened without Fate).
    pub base_sum: f64,
    pub bent_sum: f64,
}

/// How many recent rolls the debug page remembers.
const RECENT_ROLLS: usize = 200;

/// The Goddess of Fate. Saved with the world: her dice and twists.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Fate {
    /// The living world's dice.
    dice: Dice,
    pub twists: Vec<Twist>,
    next_twist_id: u32,
    /// Where things stand now (world clock seconds), for twist endings and the roll log.
    #[serde(skip)]
    now: f64,
    #[serde(skip)]
    recent: VecDeque<RollRecord>,
    #[serde(skip)]
    stats: BTreeMap<String, CategoryStats>,
}

impl Default for Fate {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Fate {
    /// A fresh Fate for a world seed.
    pub fn new(world_seed: u64) -> Self {
        Self {
            dice: Dice::new(derive_seed(world_seed, "living world")),
            twists: Vec::new(),
            next_twist_id: 1,
            now: 0.0,
            recent: VecDeque::new(),
            stats: BTreeMap::new(),
        }
    }

    /// The chance after every matching twist has bent it.
    pub fn bent_chance(&self, category: &str, chance: f64, wanted: Wanted) -> f64 {
        let doublings: f64 = self
            .twists
            .iter()
            .filter(|twist| twist.matches(category))
            .map(|twist| twist.strangeness + twist.luck * wanted.sign())
            .sum();
        bend_chance(chance, doublings)
    }

    /// Does it happen? The main way anything in the living world rolls.
    pub fn roll(&mut self, category: &str, chance: f64, wanted: Wanted) -> bool {
        let bent = self.bent_chance(category, chance, wanted);
        let result = self.dice.next_f64() < bent;
        self.remember(category, chance, bent, result);
        result
    }

    /// A d20 roll: succeeds on `needed` or higher. Fate decides success with the bent odds, then
    /// shows a die face that agrees with it (for the news: "needed 16, rolled 10").
    pub fn d20(&mut self, category: &str, needed: u8, wanted: Wanted) -> D20 {
        let needed = needed.clamp(1, 21);
        let chance = (21 - needed as i32).clamp(0, 20) as f64 / 20.0;
        let success = self.roll(category, chance, wanted);
        let rolled = if success {
            self.dice.range(needed.max(1) as u32, 20)
        } else {
            self.dice.range(1, (needed.saturating_sub(1)).clamp(1, 20) as u32)
        } as u8;
        D20 { needed, rolled, success }
    }

    pub fn add_twist(&mut self, mut twist: Twist) -> u32 {
        twist.id = self.next_twist_id;
        self.next_twist_id += 1;
        let id = twist.id;
        self.twists.push(twist);
        id
    }

    pub fn remove_twist(&mut self, id: u32) {
        self.twists.retain(|twist| twist.id != id);
    }

    fn remember(&mut self, category: &str, base: f64, bent: f64, result: bool) {
        let stats = self.stats.entry(category.to_string()).or_default();
        stats.rolls += 1;
        stats.successes += result as u64;
        stats.base_sum += base;
        stats.bent_sum += bent;
        self.recent.push_front(RollRecord { category: category.to_string(), base, bent, result, at: self.now });
        self.recent.truncate(RECENT_ROLLS);
    }
}

/// A seed from the computer's clock, for new worlds without a chosen seed.
pub fn seed_from_clock() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos() as u64);
    Dice::new(nanos).next_u64()
}

pub struct FatePlugin;

impl Plugin for FatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fate>()
            .init_resource::<FateTester>()
            .add_world_state_with("fate", save_fate, load_fate, fresh_fate)
            // First thing each tick: catch up with the clock and end twists whose time is up.
            .add_systems(FixedPreUpdate, end_expired_twists)
            .add_debug_page("Fate", fate_debug_page);
    }
}

fn end_expired_twists(clock: Res<WorldClock>, mut fate: ResMut<Fate>) {
    let now = clock.seconds;
    fate.now = now;
    if fate.twists.iter().any(|twist| twist.ends_at.is_some_and(|end| end <= now)) {
        fate.twists.retain(|twist| twist.ends_at.is_none_or(|end| end > now));
    }
}

pub(crate) fn save_fate(world: &World, folder: &WorldFolder) -> Result<String, String> {
    let fate = world.resource::<Fate>();
    folder.write_toml(fate)?;
    Ok(format!("{} twists", fate.twists.len()))
}

pub(crate) fn load_fate(world: &mut World, folder: &WorldFolder) -> Result<String, String> {
    match folder.read_toml::<Fate>()? {
        Some(fate) => {
            world.insert_resource(fate);
            Ok(String::new())
        }
        None => {
            world.insert_resource(Fate::new(folder.seed));
            Ok("not in this world, started fresh".to_string())
        }
    }
}

pub(crate) fn fresh_fate(world: &mut World, seed: u64) {
    world.insert_resource(Fate::new(seed));
}

// ── Debug page ──────────────────────────────────────────────────────────────

/// The debug page's tester. It rolls its own dice, so testing never changes the world's future.
#[derive(Resource)]
struct FateTester {
    category: String,
    chance: f64,
    wanted: Wanted,
    dice: Dice,
    last: Option<String>,
}

impl Default for FateTester {
    fn default() -> Self {
        Self {
            category: "loot.rare".to_string(),
            chance: 0.01,
            wanted: Wanted::Yes,
            dice: Dice::new(seed_from_clock()),
            last: None,
        }
    }
}

const PINK: egui::Color32 = egui::Color32::from_rgb(255, 140, 220);

fn fate_debug_page(ui: &mut egui::Ui, world: &mut World) {
    let now = world.resource::<WorldClock>().seconds;
    world.resource_scope(|world, mut fate: Mut<Fate>| {
        let mut tester = world.resource_mut::<FateTester>();
        ui.small("Every random roll in the living world goes through here. Dials: +1 doubles the odds, −1 halves them.");

        // ── Twists ──
        ui.separator();
        ui.strong(format!("Twists ({})", fate.twists.len()));
        let mut remove = None;
        for twist in &mut fate.twists {
            ui.push_id(twist.id, |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut twist.label).desired_width(150.0));
                    if ui.small_button("remove").clicked() {
                        remove = Some(twist.id);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("categories");
                    ui.add(egui::TextEdit::singleline(&mut twist.categories).desired_width(120.0));
                });
                ui.add(egui::Slider::new(&mut twist.luck, -6.0..=6.0).text("luck (for the player)"));
                ui.add(egui::Slider::new(&mut twist.strangeness, -6.0..=6.0).text("strangeness"));
                match twist.ends_at {
                    Some(end) => {
                        ui.horizontal(|ui| {
                            ui.label(format!("ends {}", describe_moment(end)));
                            if ui.small_button("never").clicked() {
                                twist.ends_at = None;
                            }
                        });
                    }
                    None => {
                        ui.horizontal(|ui| {
                            ui.label("until removed");
                            if ui.small_button("end in 1 hour").clicked() {
                                twist.ends_at = Some(now + crate::time::HOUR_SECONDS);
                            }
                            if ui.small_button("end in 1 day").clicked() {
                                twist.ends_at = Some(now + crate::time::DAY_SECONDS);
                            }
                        });
                    }
                }
                ui.add_space(4.0);
            });
        }
        if let Some(id) = remove {
            fate.remove_twist(id);
        }
        if ui.button("Add a twist").clicked() {
            fate.add_twist(Twist::default());
        }

        // ── Tester ──
        ui.separator();
        ui.strong("Try it");
        ui.horizontal(|ui| {
            ui.label("category");
            ui.add(egui::TextEdit::singleline(&mut tester.category).desired_width(120.0));
        });
        ui.add(
            egui::Slider::new(&mut tester.chance, 0.0001..=1.0)
                .logarithmic(true)
                .text("base chance"),
        );
        ui.horizontal(|ui| {
            ui.label("the player wants it:");
            ui.selectable_value(&mut tester.wanted, Wanted::Yes, "yes");
            ui.selectable_value(&mut tester.wanted, Wanted::No, "no");
            ui.selectable_value(&mut tester.wanted, Wanted::Either, "doesn't care");
        });
        let bent = fate.bent_chance(&tester.category, tester.chance, tester.wanted);
        ui.label(format!(
            "Base {}  →  after Fate {}",
            describe_chance(tester.chance),
            describe_chance(bent)
        ));
        if ui.button("Roll 1,000 times").clicked() {
            let hits = (0..1000).filter(|_| tester.dice.next_f64() < bent).count();
            tester.last = Some(format!(
                "{hits} of 1,000 happened (without Fate, about {:.0} would have).",
                tester.chance * 1000.0
            ));
        }
        if let Some(last) = &tester.last {
            ui.label(last);
        }
        ui.small("The tester rolls its own dice, so it never changes what happens in the world.");

        // ── What's been rolled ──
        ui.separator();
        ui.strong("Rolls in the world, by category (since the game started)");
        if fate.stats.is_empty() {
            ui.label("Nothing has rolled yet.");
        }
        egui::Grid::new("fate_stats").striped(true).show(ui, |ui| {
            ui.label("category");
            ui.label("rolls");
            ui.label("happened");
            ui.label("expected w/o Fate");
            ui.end_row();
            for (category, stats) in &fate.stats {
                ui.label(category);
                ui.label(stats.rolls.to_string());
                ui.label(stats.successes.to_string());
                ui.label(format!("{:.1}", stats.base_sum));
                ui.end_row();
            }
        });
        ui.add_space(4.0);
        ui.strong("Recent rolls");
        for record in fate.recent.iter().take(30) {
            let text = format!(
                "{}  {:<16} {} → {}  {}",
                describe_moment(record.at),
                record.category,
                describe_chance(record.base),
                describe_chance(record.bent),
                if record.result { "YES" } else { "no" }
            );
            if record.result {
                ui.colored_label(PINK, egui::RichText::new(text).monospace());
            } else {
                ui.monospace(text);
            }
        }
    });
}

/// 0.01 → "1.0%", 0.00012 → "0.012%"
fn describe_chance(chance: f64) -> String {
    let percent = chance * 100.0;
    if percent >= 1.0 || percent == 0.0 {
        format!("{percent:.1}%")
    } else {
        format!("{percent:.3}%")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_match_families() {
        assert!(category_matches("*", "anything"));
        assert!(category_matches("world.*", "world.raid"));
        assert!(category_matches("world.*", "world"));
        assert!(!category_matches("world.*", "worldly"));
        assert!(category_matches("loot.rare", "loot.rare"));
        assert!(!category_matches("loot.rare", "loot.common"));
        assert!(category_matches("combat.*, loot.rare", "loot.rare"));
    }

    #[test]
    fn bending_doubles_odds_and_never_passes_certain() {
        // 1 in 3 is odds of 1:2. One doubling makes it 2:2 = 50%.
        assert!((bend_chance(1.0 / 3.0, 1.0) - 0.5).abs() < 1e-9);
        assert!((bend_chance(0.5, -1.0) - 1.0 / 3.0).abs() < 1e-9);
        assert!(bend_chance(0.99, 20.0) < 1.0);
        // Impossible and certain stay that way.
        assert_eq!(bend_chance(0.0, 10.0), 0.0);
        assert_eq!(bend_chance(1.0, -10.0), 1.0);
    }

    #[test]
    fn luck_follows_what_the_player_wants() {
        let mut fate = Fate::new(1);
        fate.add_twist(Twist { luck: 2.0, categories: "combat.*".to_string(), ..Default::default() });
        let base = 0.2;
        assert!(fate.bent_chance("combat.hit", base, Wanted::Yes) > base);
        assert!(fate.bent_chance("combat.hit", base, Wanted::No) < base);
        assert_eq!(fate.bent_chance("combat.hit", base, Wanted::Either), base);
        assert_eq!(fate.bent_chance("loot.rare", base, Wanted::Yes), base);
    }

    #[test]
    fn d20_faces_agree_with_the_result() {
        let mut fate = Fate::new(9);
        for needed in 2..=20 {
            for _ in 0..50 {
                let roll = fate.d20("test", needed, Wanted::Either);
                assert_eq!(roll.success, roll.rolled >= needed, "{roll:?}");
                assert!((1..=20).contains(&roll.rolled));
            }
        }
    }

    #[test]
    fn saved_fate_rolls_the_same_future() {
        let mut fate = Fate::new(5);
        fate.roll("a", 0.5, Wanted::Either);
        let text = toml::to_string_pretty(&fate).unwrap();
        let mut loaded: Fate = toml::from_str(&text).unwrap();
        for _ in 0..20 {
            assert_eq!(fate.roll("a", 0.5, Wanted::Either), loaded.roll("a", 0.5, Wanted::Either));
        }
    }
}
