//! Terrain material — connects our block shader (`assets/shaders/terrain.wesl`) to Bevy.
//!
//! A *material* tells Bevy which shader to draw a mesh with and what extra data that shader
//! gets: here, the block texture array and a few tunable lighting/fog values.
//!
//! We deliberately switch off Bevy's realistic-lighting features (shadows, depth prepass):
//! the terrain uses its own cheap, flat, Minecraft-style shading instead.

use bevy::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

/// Our packed 12-byte vertex (see `mesher.rs` for the bit layout).
/// The number is just a unique ID so Bevy can tell this attribute apart from others.
pub const ATTRIBUTE_PACKED_VOXEL: MeshVertexAttribute =
    MeshVertexAttribute::new("PackedVoxel", 0x1A5C_A7_05, VertexFormat::Uint32x3);

const SHADER_PATH: &str = "shaders/terrain.wesl";

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TerrainMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub textures: Handle<Image>,
    #[uniform(2)]
    pub settings: TerrainShaderSettings,
    /// The world light map (shadows, valleys, relief: see `light.rs`).
    #[texture(3)]
    #[sampler(4)]
    pub light_map: Handle<Image>,
    /// The ground's slope and height in a window around you, for hill shading (see `light.rs`).
    #[texture(5)]
    #[sampler(6)]
    pub near_map: Handle<Image>,
    /// The near window's cone map (which side of a step a spot is on: see `light.rs`, "The cone
    /// map"): two plain numbers a pixel, which the shader reads and blends itself.
    #[texture(8, sample_type = "float", filterable = false)]
    pub near_cones: Handle<Image>,
    /// Kept apart from `settings` so the light (`light.rs`) and the render settings (`mod.rs`)
    /// each update their own.
    #[uniform(7)]
    pub light: TerrainLight,
}

/// How the near chunks are lit (see `light.rs`).
#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub struct TerrainLight {
    /// The world light map: x on, y shadows, z sky, w relief (strengths).
    pub light: Vec4,
    /// x the map's half width, y the light map's width (blocks).
    pub light_area: Vec4,
    /// xyz toward the sun (game axes), w hill shading strength.
    pub sun: Vec4,
    /// The window: x north and y east of its north-west corner, z its side (blocks), w on.
    pub near_area: Vec4,
    /// x height tint, y the distance slopes are measured over (blocks: the distant terrain's finest
    /// cell, so both shade alike), z sea level, w the highest a height can be.
    pub hill: Vec4,
    /// x how hill shading fades toward its limits (the distant terrain's shading fade), y foot
    /// shading, z step shadows (strengths), w the height the height tint counts up to.
    pub shade: Vec4,
    /// x how much of the lie of the land's sun and shade the steps' sides take (0 to 1).
    pub sides: Vec4,
    /// For the far copies' plain ground, lit as the far terrain is (`far_copies.rs`, terrain.wesl
    /// `far_ground`): x how much the far slopes take on the steps' shade, y the steepest slope the
    /// tables reach; and the steps' feet's shade by steepness, all round and from the sun's side
    /// (the far terrain's own tables, `distant.rs`, `Shade`).
    pub far: Vec4,
    pub foot: [Vec4; super::distant::TABLE_VEC4S],
    pub step: [Vec4; super::distant::TABLE_VEC4S],
}

/// Values the shader reads every frame. Kept to `Vec4`s, which the GPU handles most simply.
#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
pub struct TerrainShaderSettings {
    /// Brightness of faces: x = top, y = bottom, z = north/south, w = east/west.
    pub face_shade: Vec4,
    /// Fog colour (rgb), w unused.
    pub fog_color: Vec4,
    /// x = fog start distance, y = fog end distance, z = fog on (1) / off (0),
    /// w = debug view (0 normal, 1 colour each quad, 2 colour by face direction).
    pub fog_and_debug: Vec4,
    /// x = random texture rotation on (1) / off (0), yzw unused.
    pub options: Vec4,
    /// Survey view: x on, y every how many levels, z darkness, w sea level.
    pub survey: Vec4,
    /// Underground (`sky_light.rs`): x daylight on (else everything fully lit), y the test lantern's
    /// reach (blocks; 0 off), z its brightness at its middle, w how bright the sky is (0 to 1: daylight
    /// times this, a stand-in for night).
    pub underground: Vec4,
    /// Placed light (`placed_light.rs`, in the vertex): x on, y how much it flickers (0 to 1), z how
    /// fast (wavers a second).
    pub placed: Vec4,
}

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    /// Tell the GPU our vertices are just the one packed attribute.
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout
            .0
            .get_layout(&[ATTRIBUTE_PACKED_VOXEL.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
