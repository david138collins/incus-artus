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

/// Our packed 8-byte vertex (see `mesher.rs` for the bit layout).
/// The number is just a unique ID so Bevy can tell this attribute apart from others.
pub const ATTRIBUTE_PACKED_VOXEL: MeshVertexAttribute =
    MeshVertexAttribute::new("PackedVoxel", 0x1A5C_A7_05, VertexFormat::Uint32x2);

const SHADER_PATH: &str = "shaders/terrain.wesl";

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TerrainMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub textures: Handle<Image>,
    #[uniform(2)]
    pub settings: TerrainShaderSettings,
}

/// Values the shader reads every frame. Kept to `Vec4`s, which the GPU handles most simply.
#[derive(ShaderType, Clone, Copy, Debug)]
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
