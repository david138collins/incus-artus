//! The test town — TEMPORARY. Eight people on the test world, with four houses, two workplaces
//! (a workshop and a quarry), a kitchen and a tavern. Placeholder names until David's name lists.
//!
//! Layout (X north, Y east; the test world is 100 blocks across):
//! ```text
//!            quarry (35, 25)
//!   houses (28, -28) (28, -16)      kitchen (10, 30)
//!               tavern (-8, 6)
//!   workshop (-25, -30)      houses (-30, 18) (-30, 30)
//! ```

use super::People;
use super::records::{Doing, Person, PersonId, Place, PlaceId, PlaceKind, Urge};
use crate::fate::Fate;
use crate::sim::{SiteId, Spot};
use bevy::math::Vec2;

pub fn test_town(site: SiteId, centre: Vec2, fate: &mut Fate) -> People {
    let mut town = People::default();
    let mut next_place = 0;
    let mut place = |town: &mut People, name: &str, kind: PlaceKind, x: f32, y: f32| {
        next_place += 1;
        let id = PlaceId(next_place);
        town.places.push(Place { id, name: name.to_string(), kind, site, at: Spot::new(centre.x + x, centre.y + y) });
        id
    };
    let houses = [
        place(&mut town, "North House", PlaceKind::Home, 28.0, -28.0),
        place(&mut town, "North Cottage", PlaceKind::Home, 28.0, -16.0),
        place(&mut town, "South House", PlaceKind::Home, -30.0, 18.0),
        place(&mut town, "South Cottage", PlaceKind::Home, -30.0, 30.0),
    ];
    let workshop = place(&mut town, "Workshop", PlaceKind::Workplace, -25.0, -30.0);
    let quarry = place(&mut town, "Quarry", PlaceKind::Workplace, 35.0, 25.0);
    place(&mut town, "Kitchen", PlaceKind::Kitchen, 10.0, 30.0);
    place(&mut town, "Tavern", PlaceKind::Tavern, -8.0, 6.0);

    for number in 0..8u32 {
        let home = houses[number as usize / 2];
        let work = if number % 2 == 0 { workshop } else { quarry };
        let at = super::behaviour::standing_spot(
            town.places.iter().find(|p| p.id == home).unwrap().at,
            PersonId(number + 1),
        );
        // Start everyone part-way through their day, so they don't all move as one.
        let mut charges = [0.0; 5];
        for urge in Urge::ALL {
            charges[urge.index()] = (fate.noise() * 900.0) as f32;
        }
        town.people.push(Person {
            id: PersonId(number + 1),
            name: format!("Villager {}", number + 1),
            site,
            home,
            work: Some(work),
            at,
            charges,
            pressures: Vec::new(),
            doing: Doing::Idle,
            friends: Vec::new(),
        });
    }
    town
}
