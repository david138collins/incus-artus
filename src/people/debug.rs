//! The "People" debug page (everyone's urges, thresholds and the pressures on them), and the
//! floating labels over the people cubes.

use super::behaviour::{self, is_night};
use super::meetings::start_a_wave;
use super::records::{Doing, Person, Place, Urge};
use super::{MoodWave, People, PeopleSettings};
use crate::camera::FlyCamera;
use crate::fate::Fate;
use crate::sim::{SimLevel, SimWorld};
use crate::time::{HOUR_SECONDS, TimeControl, WorldClock, describe_moment, describe_span};
use crate::world::TestWorldSettings;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

const PINK: egui::Color32 = egui::Color32::from_rgb(255, 140, 220);
const MAGENTA: egui::Color32 = egui::Color32::from_rgb(255, 0, 255);

fn place_name(places: &[Place], id: Option<super::records::PlaceId>) -> String {
    id.and_then(|id| places.iter().find(|p| p.id == id)).map_or("somewhere about".to_string(), |p| p.name.clone())
}

/// "going to eat (Kitchen)", "chatting with Villager 3"...
pub(super) fn describe_doing(person: &Person, people: &People) -> String {
    match &person.doing {
        Doing::Idle => "standing about".to_string(),
        Doing::Going { urge, place, .. } => format!("going to {urge} ({})", place_name(&people.places, *place)),
        Doing::Satisfying { urge, place } => {
            let verb = match urge {
                Urge::Work => "working",
                Urge::Eat => "eating",
                Urge::Sleep => "sleeping",
                Urge::Socialise => "socialising",
                Urge::Wander => "wandering",
            };
            format!("{verb} ({})", place_name(&people.places, *place))
        }
        Doing::Chatting { with, .. } => {
            let name = people.people.iter().find(|p| p.id == *with).map_or("someone", |p| p.name.as_str());
            format!("chatting with {name}")
        }
    }
}

pub(super) fn people_debug_page(ui: &mut egui::Ui, world: &mut World) {
    let now = world.resource::<WorldClock>().seconds;
    let night = is_night(world.resource::<PeopleSettings>(), now);
    ui.label(format!("World clock: {}{}", describe_moment(now), if night { "  (night)" } else { "" }));
    ui.small(
        "Each urge charges up; when its charge reaches its threshold it can become the goal (the fullest wins). \
         Pressures push thresholds down. Cubes: work blue, eat green, sleep purple, socialise magenta, \
         wander white, chatting yellow, standing about grey.",
    );
    let level = {
        let people = world.resource::<People>();
        let site = people.people.first().map(|p| p.site);
        world.resource::<SimWorld>().sites.iter().find(|s| Some(s.id) == site).map(|s| (s.name.clone(), s.level))
    };
    if let Some((name, level)) = &level {
        ui.label(format!("{name} is {level}{}", if *level == SimLevel::Local { ": people move every tick" } else { ": people move on the books" }));
    }

    // ── Settings ──
    egui::CollapsingHeader::new("Settings").show(ui, |ui| {
        let mut settings = world.resource_mut::<PeopleSettings>();
        for urge in Urge::ALL {
            ui.strong(urge.to_string());
            let rule = settings.rule_mut(urge);
            ui.add(egui::Slider::new(&mut rule.rate_per_hour, 0.0..=1000.0).text("charge per hour"));
            ui.add(egui::Slider::new(&mut rule.threshold, 100.0..=5000.0).text("threshold"));
            ui.add(egui::Slider::new(&mut rule.drain_per_hour, 10.0..=5000.0).logarithmic(true).text("satisfied per hour"));
        }
        ui.separator();
        ui.add(egui::Slider::new(&mut settings.interrupt_ratio, 1.0..=4.0).text("interrupt when another urge is × fuller"));
        ui.add(egui::Slider::new(&mut settings.night_sleep_pressure, 0.0..=1000.0).text("night presses sleep down by"));
        ui.add(egui::Slider::new(&mut settings.night_from_hour, 0.0..=24.0).text("night from (hour)"));
        ui.add(egui::Slider::new(&mut settings.night_to_hour, 0.0..=24.0).text("night until (hour)"));
        ui.add(egui::Slider::new(&mut settings.asleep_charge_factor, 0.0..=1.0).text("asleep, other urges charge ×"));
        ui.add(egui::Slider::new(&mut settings.randomness, 0.0..=1.0).text("random wobble on charging (±)"));
        ui.add(egui::Slider::new(&mut settings.walk_speed, 0.2..=6.0).text("walking speed (blocks/s)"));
        ui.separator();
        ui.add(egui::Slider::new(&mut settings.chat_distance, 0.5..=10.0).text("notice each other within (blocks)"));
        ui.add(egui::Slider::new(&mut settings.chat_chance, 0.0..=1.0).text("chance to notice (more for friends)"));
        ui.add(egui::Slider::new(&mut settings.chat_pressure, 0.0..=1000.0).text("noticing presses socialise down by"));
        ui.add(egui::Slider::new(&mut settings.chat_minutes, 1.0..=60.0).text("a chat lasts (in-game minutes)"));
        ui.add(egui::Slider::new(&mut settings.chat_satisfaction, 0.0..=1000.0).text("a chat satisfies"));
        ui.add(egui::Slider::new(&mut settings.chat_friendship, 0.0..=0.5).text("a chat adds friendship"));
        ui.separator();
        ui.add(egui::Slider::new(&mut settings.weirdness_per_hour, 0.0..=1.0).text("waves of weirdness: chance per hour"));
        ui.add(egui::Slider::new(&mut settings.weirdness_pressure, 0.0..=2000.0).text("a wave presses its urge down by"));
        ui.checkbox(&mut settings.show_labels, "names over the cubes");
        if ui.button("Reset to defaults").clicked() {
            *settings = PeopleSettings::default();
        }
    });

    if ui.button("A wave of weirdness, now").clicked() {
        let settings = world.resource::<PeopleSettings>().clone();
        let tick = world.resource::<TimeControl>().tick();
        let (what, people) = world.resource_scope(|world, mut people: Mut<People>| {
            let mut fate = world.resource_mut::<Fate>();
            start_a_wave(&mut people.people, &settings, now, &mut fate)
        });
        world.write_message(MoodWave { what, people, tick });
    }

    // ── Everyone ──
    ui.separator();
    let settings = world.resource::<PeopleSettings>().clone();
    let mut press: Option<(usize, Urge)> = None;
    let mut unpress: Option<(usize, usize)> = None;
    {
        let people = world.resource::<People>();
        for (index, person) in people.people.iter().enumerate() {
            let header = format!("{}: {}", person.name, describe_doing(person, people));
            egui::CollapsingHeader::new(header).id_salt(("person", person.id.0)).show(ui, |ui| {
                for urge in Urge::ALL {
                    let line = behaviour::threshold(person, urge, &settings, now);
                    let charge = person.charge(urge);
                    let base = settings.rule(urge).threshold;
                    let pushed = if line < base { format!(" (pushed down from {base:.0})") } else { String::new() };
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::ProgressBar::new((charge / line).min(1.0))
                                .desired_width(150.0)
                                .text(format!("{urge} {charge:.0} / {line:.0}")),
                        );
                        if charge >= line {
                            ui.colored_label(MAGENTA, "FULL");
                        }
                        ui.small(pushed);
                        if ui.small_button("press").on_hover_text("Press this urge's threshold down by 500 for an hour").clicked() {
                            press = Some((index, urge));
                        }
                    });
                }
                if night {
                    ui.small(format!("night presses sleep down by {:.0}", settings.night_sleep_pressure));
                }
                for (number, pressure) in person.pressures.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let ends = pressure.ends_at.map_or("until removed".to_string(), |end| format!("for {}", describe_span(end - now)));
                        ui.colored_label(PINK, format!("pressure on {}: −{:.0}, {} ({ends})", pressure.urge, pressure.amount, pressure.why));
                        if ui.small_button("remove").clicked() {
                            unpress = Some((index, number));
                        }
                    });
                }
                if person.friends.is_empty() {
                    ui.small("No friends yet.");
                } else {
                    let friends: Vec<String> = person
                        .friends
                        .iter()
                        .map(|f| {
                            let name = people.people.iter().find(|p| p.id == f.with).map_or("?", |p| p.name.as_str());
                            format!("{name} {:.2}", f.friendship)
                        })
                        .collect();
                    ui.small(format!("Friends: {}", friends.join(", ")));
                }
            });
        }
    }
    let mut people = world.resource_mut::<People>();
    if let Some((index, urge)) = press {
        behaviour::press(&mut people.people[index], urge, 500.0, "pressed on the debug page".to_string(), Some(now + HOUR_SECONDS));
    }
    if let Some((index, number)) = unpress {
        people.people[index].pressures.remove(number);
    }
}

/// Names and what they're doing, floating over the cubes (and place names on the ground).
pub(super) fn draw_labels(
    mut contexts: EguiContexts,
    people: Res<People>,
    settings: Res<PeopleSettings>,
    sim: Res<SimWorld>,
    terrain: Res<TestWorldSettings>,
    cameras: Query<(&Camera, &GlobalTransform, &FlyCamera)>,
) -> Result {
    if !settings.show_labels {
        return Ok(());
    }
    let Ok((camera, transform, fly)) = cameras.single() else {
        return Ok(());
    };
    let local = |site| sim.sites.iter().any(|s| s.id == site && s.level == SimLevel::Local);
    let ground = terrain.grass_z as f32 + 1.0;
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("people_labels")));
    let label = |at: Vec3, text: String, colour: egui::Color32, size: f32| {
        if at.distance(fly.position) > 70.0 {
            return;
        }
        if let Ok(spot) = camera.world_to_viewport(transform, crate::coords::game_to_bevy(at)) {
            painter.text(egui::pos2(spot.x, spot.y), egui::Align2::CENTER_BOTTOM, text, egui::FontId::proportional(size), colour);
        }
    };
    for place in people.places.iter().filter(|p| local(p.site)) {
        label(place.at.vec().extend(ground + 0.3), place.name.clone(), egui::Color32::GRAY, 12.0);
    }
    for person in people.people.iter().filter(|p| local(p.site)) {
        let text = format!("{}\n{}", person.name, describe_doing(person, &people));
        label(person.at.vec().extend(ground + 1.8), text, PINK, 13.0);
    }
    Ok(())
}
