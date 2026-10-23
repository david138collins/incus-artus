//! Meetings and moods — opportunities that present themselves through chance.
//!
//! **Meeting someone** is an opportunity: when two people pass close by, there's a chance (through
//! Fate, `social.chat`) that they notice each other. That **presses down** their socialise
//! threshold for a few minutes. If both are then charged enough, they stop and chat: it satisfies
//! some of their need to socialise, and they become a little friendlier. People who pass each
//! other every day become friends.
//!
//! In the **bookkeeping** (when the settlement isn't local) nobody walks past anybody, so instead,
//! now and then, each pair rolls for "did they cross paths and chat?", likelier for people who
//! work or live together.
//!
//! **Waves of weirdness** (`people.mood`) come by once in a while and press one urge down for a
//! share of the people at random, just to stir things up.

use super::PeopleSettings;
use super::behaviour::press;
use super::records::{Doing, Person, PersonId, Urge};
use crate::fate::{Fate, Wanted};
use crate::time::{DAY_SECONDS, HOUR_SECONDS};
use std::collections::HashMap;

/// One in-game minute, in world seconds.
pub const MINUTE: f64 = HOUR_SECONDS / 60.0;

pub enum MeetingNews {
    Chatted { a: String, b: String, friendship: f32, bookkeeping: bool },
    WalkedOn { a: String, b: String },
}

/// People passing close by each other, while the settlement is local.
pub fn meetings_in_person(
    people: &mut [Person],
    settings: &PeopleSettings,
    now: f64,
    recently_met: &mut HashMap<(PersonId, PersonId), f64>,
    fate: &mut Fate,
) -> Vec<MeetingNews> {
    let mut news = Vec::new();
    for i in 0..people.len() {
        for j in (i + 1)..people.len() {
            let (a, b) = (&people[i], &people[j]);
            let walking = |p: &Person| matches!(p.doing, Doing::Going { .. } | Doing::Idle);
            if !walking(a) || !walking(b) || a.at.vec().distance(b.at.vec()) > settings.chat_distance {
                continue;
            }
            let pair = (a.id, b.id);
            if recently_met.get(&pair).is_some_and(|&until| until > now) {
                continue;
            }
            // Don't notice the same person again for an hour.
            recently_met.insert(pair, now + HOUR_SECONDS);

            let friendship = a.friendship(b.id).max(b.friendship(a.id));
            let chance = (settings.chat_chance + 0.4 * friendship as f64).min(0.95);
            if !fate.roll("social.chat", chance, Wanted::Either) {
                continue;
            }
            // They notice each other: an opportunity to chat presses their thresholds down.
            let ends = Some(now + 5.0 * MINUTE);
            let (a_name, b_name) = (a.name.clone(), b.name.clone());
            press(&mut people[i], Urge::Socialise, settings.chat_pressure, format!("ran into {b_name}"), ends);
            press(&mut people[j], Urge::Socialise, settings.chat_pressure, format!("ran into {a_name}"), ends);
            let keen = |p: &Person| super::behaviour::fullness(p, Urge::Socialise, settings, now) >= 1.0;
            if keen(&people[i]) && keen(&people[j]) {
                let seconds_left = (settings.chat_minutes as f64 * MINUTE) as f32;
                let (id_a, id_b) = (people[i].id, people[j].id);
                people[i].doing = Doing::Chatting { with: id_b, seconds_left };
                people[j].doing = Doing::Chatting { with: id_a, seconds_left };
                let friendship = people[i].befriend(id_b, settings.chat_friendship);
                people[j].befriend(id_a, settings.chat_friendship);
                news.push(MeetingNews::Chatted { a: a_name, b: b_name, friendship, bookkeeping: false });
            } else {
                news.push(MeetingNews::WalkedOn { a: a_name, b: b_name });
            }
        }
    }
    news
}

/// The bookkeeping version: over `seconds`, did each pair cross paths and chat?
pub fn meetings_on_the_books(people: &mut [Person], settings: &PeopleSettings, seconds: f64, fate: &mut Fate) -> Vec<MeetingNews> {
    let days = seconds / DAY_SECONDS;
    let mut news = Vec::new();
    for i in 0..people.len() {
        for j in (i + 1)..people.len() {
            let (a, b) = (&people[i], &people[j]);
            // How likely they are to cross paths on a given day.
            let crossing = if a.work.is_some() && a.work == b.work {
                0.6
            } else if a.home == b.home {
                0.4
            } else {
                0.15
            };
            let friendship = a.friendship(b.id).max(b.friendship(a.id));
            let per_day = crossing * (settings.chat_chance + 0.4 * friendship as f64).min(0.95);
            let chance = 1.0 - (1.0 - per_day).powf(days);
            if fate.roll("social.chat", chance, Wanted::Either) {
                let (id_a, id_b) = (a.id, b.id);
                let (a_name, b_name) = (a.name.clone(), b.name.clone());
                let friendship = people[i].befriend(id_b, settings.chat_friendship);
                people[j].befriend(id_a, settings.chat_friendship);
                for k in [i, j] {
                    let charge = people[k].charge_mut(Urge::Socialise);
                    *charge = (*charge - settings.chat_satisfaction).max(0.0);
                }
                news.push(MeetingNews::Chatted { a: a_name, b: b_name, friendship, bookkeeping: true });
            }
        }
    }
    news
}

/// Labels for the waves (placeholders until David writes moods).
fn wave_label(urge: Urge) -> &'static str {
    match urge {
        Urge::Work => "a fit of industriousness",
        Urge::Eat => "a sudden hunger",
        Urge::Sleep => "a drowsy spell",
        Urge::Socialise => "a chatty mood",
        Urge::Wander => "a restless mood",
    }
}

/// A wave of weirdness: one urge pressed down for a share of the people, for a few hours.
/// Returns (label, how many people) if one came by.
pub fn maybe_a_wave(people: &mut [Person], settings: &PeopleSettings, now: f64, seconds: f64, fate: &mut Fate) -> Option<(String, usize)> {
    let hours = seconds / HOUR_SECONDS;
    let chance = 1.0 - (1.0 - settings.weirdness_per_hour).powf(hours);
    if people.is_empty() || !fate.roll("people.mood", chance, Wanted::Either) {
        return None;
    }
    Some(start_a_wave(people, settings, now, fate))
}

/// Start a wave now (also the debug page's button).
pub fn start_a_wave(people: &mut [Person], settings: &PeopleSettings, now: f64, fate: &mut Fate) -> (String, usize) {
    let urge = Urge::ALL[fate.pick(Urge::ALL.len())];
    let label = wave_label(urge);
    let share = 0.3 + 0.4 * fate.noise();
    let hours = 1.0 + 2.0 * fate.noise();
    let mut touched = 0;
    for person in people.iter_mut() {
        if fate.noise() < share {
            press(person, urge, settings.weirdness_pressure, label.to_string(), Some(now + hours * HOUR_SECONDS));
            touched += 1;
        }
    }
    (format!("{label} ({urge})"), touched)
}
