//! Stand-ins — TEMPORARY. Magenta cubes that show what the local simulation would have made real:
//! when a place goes Local, a cube appears for each of its fighters (and one per ten people);
//! when a caravan is met, a wagon appears and rolls along its route. When they collapse back,
//! the cubes vanish into numbers again.
//!
//! This is what "expand" and "collapse" will do for real later (villagers, wagons and donkeys).
//! It keeps the cubes in step with the records every tick, so it never misses a change.

use super::records::{SimLevel, SimThing};
use super::{SimSettings, SimWorld, journey};
use crate::time::SimPosition;
use crate::world::TestWorldSettings;
use bevy::prelude::*;
use std::collections::HashMap;

pub(super) struct StandInPlugin;

impl Plugin for StandInPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, make_assets)
            .add_systems(FixedUpdate, keep_stand_ins_in_step.after(super::run_world_sim));
    }
}

#[derive(Component)]
struct StandIn {
    of: SimThing,
    number: u32,
}

#[derive(Resource)]
struct StandInAssets {
    person: Handle<Mesh>,
    wagon: Handle<Mesh>,
    magenta: Handle<StandardMaterial>,
    pink: Handle<StandardMaterial>,
}

fn make_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let flat = |color: Color| StandardMaterial { base_color: color, unlit: true, ..default() };
    commands.insert_resource(StandInAssets {
        // Bevy's axes: x = east, y = up, z = south. A person is 0.6 wide and 1.5 tall.
        person: meshes.add(Cuboid::new(0.6, 1.5, 0.6)),
        wagon: meshes.add(Cuboid::new(1.6, 1.4, 3.0)),
        magenta: materials.add(flat(Color::srgb(1.0, 0.0, 1.0))),
        pink: materials.add(flat(Color::srgb(1.0, 0.55, 0.85))),
    });
}

/// What should be standing where: one entry per cube.
fn wanted_stand_ins(sim: &SimWorld, ground: f32) -> HashMap<(SimThing, u32), (Vec3, bool)> {
    let mut wanted = HashMap::new();
    for site in sim.sites.iter().filter(|site| site.level == SimLevel::Local) {
        let people = site.condition.fighters.floor() as u32 + (site.condition.population / 10.0).floor() as u32;
        for number in 0..people.min(60) {
            // A sunflower spiral spreads them evenly over the footprint.
            let angle = number as f32 * 2.399_963;
            let distance = site.radius * ((number as f32 + 0.5) / people.min(60) as f32).sqrt();
            let at = site.at.vec() + Vec2::new(angle.cos(), angle.sin()) * distance;
            let fighter = number < site.condition.fighters.floor() as u32;
            wanted.insert((SimThing::Site(site.id), number), (at.extend(ground + 0.75), fighter));
        }
    }
    for trip in sim.journeys.iter().filter(|trip| trip.level == SimLevel::Local) {
        if let Some(at) = journey::position(trip, &sim.sites) {
            wanted.insert((SimThing::Journey(trip.id), 0), (at.extend(ground + 0.7), false));
        }
    }
    wanted
}

fn keep_stand_ins_in_step(
    mut commands: Commands,
    sim: Res<SimWorld>,
    settings: Res<SimSettings>,
    terrain: Res<TestWorldSettings>,
    assets: Option<Res<StandInAssets>>,
    mut existing: Query<(Entity, &StandIn, &mut SimPosition)>,
) {
    let Some(assets) = assets else {
        return;
    };
    let ground = terrain.grass_z as f32 + 1.0;
    let mut wanted = if settings.show_stand_ins { wanted_stand_ins(&sim, ground) } else { HashMap::new() };

    // Move the ones that should stay, remove the rest.
    for (entity, stand_in, mut position) in &mut existing {
        match wanted.remove(&(stand_in.of, stand_in.number)) {
            Some((at, _)) => position.current = at,
            None => commands.entity(entity).despawn(),
        }
    }
    // Make the missing ones.
    for ((of, number), (at, fighter)) in wanted {
        let (mesh, material) = match of {
            SimThing::Journey(_) => (assets.wagon.clone(), assets.pink.clone()),
            SimThing::Site(_) if fighter => (assets.person.clone(), assets.magenta.clone()),
            SimThing::Site(_) => (assets.person.clone(), assets.pink.clone()),
        };
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(crate::coords::game_to_bevy(at)),
            SimPosition::new(at),
            StandIn { of, number },
        ));
    }
}
