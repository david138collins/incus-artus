//! The "Sim" debug page (the world simulation as a spreadsheet) and the on-screen notice of what's
//! local around you.

use super::records::{HookKind, SimLevel};
use super::{Presence, SimSettings, SimWorld, SiteHookRequest, journey, skip_ahead};
use crate::time::{DAY_SECONDS, HOUR_SECONDS, WorldClock, describe_moment, describe_span};
use bevy::prelude::*;
use crate::ui::EguiContexts;

const PINK: egui::Color32 = egui::Color32::from_rgb(255, 140, 220);
const MAGENTA: egui::Color32 = egui::Color32::from_rgb(255, 0, 255);

fn level_colour(level: SimLevel) -> egui::Color32 {
    match level {
        SimLevel::Local => MAGENTA,
        SimLevel::Rich => PINK,
        SimLevel::Thin => egui::Color32::GRAY,
    }
}

pub(super) fn sim_debug_page(ui: &mut egui::Ui, world: &mut World) {
    let now = world.resource::<WorldClock>().seconds;
    let presences: Vec<Vec2> = world.query::<&Presence>().iter(world).map(|p| p.0.truncate()).collect();
    let nearest = |at: Vec2| presences.iter().map(|p| p.distance(at)).fold(f32::INFINITY, f32::min);

    ui.label(format!("World clock: {}", describe_moment(now)));
    ui.small("Thin = bookkeeping, rarely updated. Rich = involved with the player, updated often. Local = the full simulation.");

    // ── Skip ahead ──
    let mut skip = None;
    ui.horizontal(|ui| {
        ui.label("Skip ahead:");
        for (label, seconds) in [("1 hour", HOUR_SECONDS), ("1 day", DAY_SECONDS), ("7 days", 7.0 * DAY_SECONDS), ("1 year", 28.0 * DAY_SECONDS)] {
            if ui.button(label).clicked() {
                skip = Some(seconds);
            }
        }
    });
    if let Some(seconds) = skip {
        let started = std::time::Instant::now();
        skip_ahead(world, seconds);
        let millis = started.elapsed().as_secs_f64() * 1000.0;
        world.resource_mut::<SimPageState>().last_skip = Some(format!("Skipped {} in {millis:.1} ms.", describe_span(seconds)));
    }
    if let Some(text) = &world.resource::<SimPageState>().last_skip {
        ui.small(text);
    }

    // ── Settings ──
    ui.separator();
    egui::CollapsingHeader::new("Settings").show(ui, |ui| {
        let mut settings = world.resource_mut::<SimSettings>();
        ui.add(crate::debug::Slider::new(&mut settings.rich_every_hours, 0.1..=12.0).text("rich: update every (in-game hours)"));
        ui.add(crate::debug::Slider::new(&mut settings.thin_every_days, 0.1..=7.0).text("thin: update every (in-game days)"));
        ui.add(crate::debug::Slider::new(&mut settings.local_margin, 0.0..=256.0).text("local: within footprint + (blocks)"));
        ui.add(crate::debug::Slider::new(&mut settings.leave_extra, 0.0..=128.0).text("stop being local: further by (blocks)"));
        ui.add(crate::debug::Slider::new(&mut settings.encounter_range, 10.0..=1000.0).text("journeys can be met within (blocks)"));
        ui.add(crate::debug::Slider::new(&mut settings.encounter_check_minutes, 1.0..=120.0).text("\"are they here?\" roll every (in-game minutes)"));
        ui.add(crate::debug::Slider::new(&mut settings.updates_per_tick, 1..=64).text("updates per tick, at most"));
        ui.checkbox(&mut settings.show_stand_ins, "show magenta stand-ins for local things");
        if ui.button("Reset to defaults").clicked() {
            *settings = SimSettings::default();
        }
    });

    let mut requests = Vec::new();
    {
        let sim = world.resource::<SimWorld>();
        ui.small(format!("Last tick updated {} records in {:.3} ms.", sim.last_updates, sim.last_millis));

        // ── Places ──
        ui.separator();
        ui.strong(format!("Places ({})", sim.sites.len()));
        for site in &sim.sites {
            ui.push_id(("site", site.id.0), |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(level_colour(site.level), format!("[{}]", site.level));
                    ui.strong(&site.name);
                    ui.label(format!("({}, {:.0} blocks away)", site.kind.replace('_', " "), nearest(site.at.vec())));
                });
                let c = &site.condition;
                ui.monospace(format!(
                    "people {:>4.0}  fighters {:>3.0}  wealth {:>5.0}  food {:>5.0}",
                    c.population, c.fighters, c.wealth, c.food
                ));
                let since = match site.level {
                    SimLevel::Local => "the local simulation has it".to_string(),
                    _ => format!(
                        "last updated {} ago, next in {}",
                        describe_span(now - site.last_update),
                        describe_span(site.next_update - now)
                    ),
                };
                ui.small(since);
                for problem in &site.problems {
                    ui.colored_label(PINK, format!("problem: {problem}"));
                }
                for hook in &site.hooks {
                    ui.horizontal(|ui| {
                        let ends = hook.expires_at.map_or("until removed".to_string(), |end| format!("ends in {}", describe_span(end - now)));
                        ui.label(format!("hook: {} \"{}\" ({ends})", hook.kind, hook.label));
                        if ui.small_button("remove").clicked() {
                            requests.push(SiteHookRequest::Remove { site: site.id, hook: hook.id });
                        }
                    });
                }
                ui.horizontal(|ui| {
                    if ui.small_button("take a quest here (1 day)").clicked() {
                        requests.push(SiteHookRequest::Add {
                            site: site.id,
                            kind: HookKind::Quest,
                            label: "test quest".to_string(),
                            lasts: Some(DAY_SECONDS),
                        });
                    }
                    if ui.small_button("take residence").clicked() {
                        requests.push(SiteHookRequest::Add {
                            site: site.id,
                            kind: HookKind::Residence,
                            label: "lives here".to_string(),
                            lasts: None,
                        });
                    }
                });
                ui.add_space(6.0);
            });
        }

        // ── Journeys ──
        ui.separator();
        ui.strong(format!("Journeys ({})", sim.journeys.len()));
        for trip in &sim.journeys {
            let at = journey::estimate(trip, &sim.sites, now - trip.last_update);
            ui.horizontal(|ui| {
                ui.colored_label(level_colour(trip.level), format!("[{}]", trip.level));
                ui.strong(&trip.name);
                if let Some(at) = at {
                    ui.label(format!("({:.0} blocks away, at {:.0} N {:.0} E)", nearest(at), at.x, at.y));
                }
            });
            let mut moved = trip.clone();
            journey::advance(&mut moved, &sim.sites, now - trip.last_update);
            ui.small(format!(
                "{}; carrying {:.0} wealth of goods; position last worked out {} ago",
                journey::describe(&moved, &sim.sites),
                trip.cargo,
                describe_span(now - trip.last_update)
            ));
            ui.add_space(6.0);
        }
    }
    if !requests.is_empty() {
        ui.small("(hook changes happen on the next tick: they wait while time is paused)");
    }
    for request in requests {
        world.write_message(request);
    }
}

#[derive(Resource, Default)]
pub(super) struct SimPageState {
    last_skip: Option<String>,
}

/// Placeholder notice: what's in the local simulation around you right now.
pub(super) fn draw_local_notice(mut contexts: EguiContexts, sim: Res<SimWorld>) -> Result {
    let local: Vec<&str> = sim
        .sites
        .iter()
        .filter(|site| site.level == SimLevel::Local)
        .map(|site| site.name.as_str())
        .chain(sim.journeys.iter().filter(|trip| trip.level == SimLevel::Local).map(|trip| trip.name.as_str()))
        .collect();
    if local.is_empty() {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("local_notice")));
    let screen = ctx.content_rect();
    let text = format!("LOCAL: {}", local.join(", "));
    let width = 40.0 + text.len() as f32 * 7.5;
    let rect = egui::Rect::from_center_size(egui::pos2(screen.center().x, screen.top() + 56.0), egui::vec2(width, 24.0));
    painter.rect(rect, 0.0, egui::Color32::BLACK, egui::Stroke::new(1.0, MAGENTA), egui::StrokeKind::Inside);
    painter.text(rect.center(), egui::Align2::CENTER_CENTER, text, egui::FontId::proportional(14.0), PINK);
    Ok(())
}
