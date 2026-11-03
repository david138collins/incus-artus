//! Stand-ins for people — TEMPORARY coloured cubes, one per person, coloured by what they're doing,
//! plus a flat marker for each place. Shown while their settlement is local.
//!
//! Colours:  work = blue · eat = green · sleep = purple · socialise = magenta · wander = white ·
//!           chatting = yellow · standing about = grey
//! Places:   home = brown · workplace = dark blue · kitchen = dark green · tavern = dark magenta

use super::People;
use super::records::{Doing, PersonId, PlaceId, PlaceKind, Urge};
use crate::sim::{SimLevel, SimWorld};
use crate::time::SimPosition;
use crate::world::TestWorldSettings;
use bevy::prelude::*;
use std::collections::HashMap;

pub(super) struct PeopleStandInPlugin;

impl Plugin for PeopleStandInPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, make_assets)
            .add_systems(FixedUpdate, keep_in_step.after(super::live));
    }
}

#[derive(Component)]
struct PersonCube(PersonId);

#[derive(Component)]
struct PlaceMarker(PlaceId);

/// What colour a person is.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Look {
    Urge(Urge),
    Chatting,
    Idle,
}

fn look(doing: &Doing) -> Look {
    match doing {
        Doing::Chatting { .. } => Look::Chatting,
        Doing::Idle => Look::Idle,
        other => other.urge().map_or(Look::Idle, Look::Urge),
    }
}

#[derive(Resource)]
struct PeopleAssets {
    body: Handle<Mesh>,
    marker: Handle<Mesh>,
    looks: HashMap<Look, Handle<StandardMaterial>>,
    places: HashMap<u8, Handle<StandardMaterial>>,
}

fn place_code(kind: PlaceKind) -> u8 {
    match kind {
        PlaceKind::Home => 0,
        PlaceKind::Workplace => 1,
        PlaceKind::Kitchen => 2,
        PlaceKind::Tavern => 3,
    }
}

fn make_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mut flat = |r: f32, g: f32, b: f32| {
        materials.add(StandardMaterial { base_color: Color::srgb(r, g, b), unlit: true, ..default() })
    };
    let looks = HashMap::from([
        (Look::Urge(Urge::Work), flat(0.2, 0.45, 1.0)),
        (Look::Urge(Urge::Eat), flat(0.2, 0.9, 0.3)),
        (Look::Urge(Urge::Sleep), flat(0.45, 0.25, 0.75)),
        (Look::Urge(Urge::Socialise), flat(1.0, 0.0, 1.0)),
        (Look::Urge(Urge::Wander), flat(0.92, 0.92, 0.92)),
        (Look::Chatting, flat(1.0, 0.9, 0.1)),
        (Look::Idle, flat(0.5, 0.5, 0.5)),
    ]);
    let places = HashMap::from([
        (0, flat(0.45, 0.3, 0.15)),
        (1, flat(0.1, 0.2, 0.5)),
        (2, flat(0.1, 0.4, 0.15)),
        (3, flat(0.45, 0.0, 0.45)),
    ]);
    commands.insert_resource(PeopleAssets {
        // Bevy's axes: x = east, y = up, z = south.
        body: meshes.add(Cuboid::new(0.6, 1.5, 0.6)),
        marker: meshes.add(Cuboid::new(3.0, 0.1, 3.0)),
        looks,
        places,
    });
}

#[allow(clippy::too_many_arguments)]
fn keep_in_step(
    mut commands: Commands,
    people: Res<People>,
    sim: Res<SimWorld>,
    terrain: Res<TestWorldSettings>,
    assets: Option<Res<PeopleAssets>>,
    mut cubes: Query<(Entity, &PersonCube, &mut SimPosition, &mut MeshMaterial3d<StandardMaterial>)>,
    markers: Query<(Entity, &PlaceMarker)>,
    origin: Res<crate::coords::RenderOrigin>,
) {
    let Some(assets) = assets else {
        return;
    };
    let ground = terrain.grass_z as f32 + 1.0;
    let local = |site| sim.sites.iter().any(|s| s.id == site && s.level == SimLevel::Local);

    // People.
    let mut wanted: HashMap<PersonId, (Vec3, Look)> = people
        .people
        .iter()
        .filter(|p| local(p.site))
        .map(|p| (p.id, (p.at.vec().extend(ground + 0.75), look(&p.doing))))
        .collect();
    for (entity, cube, mut position, mut material) in &mut cubes {
        match wanted.remove(&cube.0) {
            Some((at, look)) => {
                position.current = at;
                let colour = &assets.looks[&look];
                if material.0 != *colour {
                    material.0 = colour.clone();
                }
            }
            None => commands.entity(entity).despawn(),
        }
    }
    for (id, (at, look)) in wanted {
        commands.spawn((
            Mesh3d(assets.body.clone()),
            MeshMaterial3d(assets.looks[&look].clone()),
            Transform::from_translation(origin.to_bevy(at)),
            SimPosition::new(at),
            PersonCube(id),
        ));
    }

    // Places.
    let mut wanted_places: HashMap<PlaceId, (Vec3, PlaceKind)> = people
        .places
        .iter()
        .filter(|p| local(p.site))
        .map(|p| (p.id, (p.at.vec().extend(ground + 0.05), p.kind)))
        .collect();
    for (entity, marker) in &markers {
        if wanted_places.remove(&marker.0).is_none() {
            commands.entity(entity).despawn();
        }
    }
    for (id, (at, kind)) in wanted_places {
        commands.spawn((
            Mesh3d(assets.marker.clone()),
            MeshMaterial3d(assets.places[&place_code(kind)].clone()),
            Transform::from_translation(origin.to_bevy(at)),
            PlaceMarker(id),
        ));
    }
}
