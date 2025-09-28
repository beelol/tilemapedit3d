#import bevy_pbr::{
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{FragmentOutput, VertexOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{FragmentOutput, VertexOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif
#ifdef MESHLET_MESH_MATERIAL_PASS
#import bevy_pbr::meshlet_visibility_buffer_resolve::resolve_vertex_output
#endif

const EPSILON: f32 = 1e-5;

struct TerrainSplatSettings {
    map_size: vec2<f32>,
    uv_scale: f32,
    _padding: f32,
};

@group(2) @binding(100) var<uniform> terrain_settings: TerrainSplatSettings;
@group(2) @binding(101) var terrain_splat_texture: texture_2d<f32>;
@group(2) @binding(102) var terrain_splat_sampler: sampler;
@group(2) @binding(103) var terrain_layer0_texture: texture_2d<f32>;
@group(2) @binding(104) var terrain_layer0_sampler: sampler;
@group(2) @binding(105) var terrain_layer1_texture: texture_2d<f32>;
@group(2) @binding(106) var terrain_layer1_sampler: sampler;
@group(2) @binding(107) var terrain_layer2_texture: texture_2d<f32>;
@group(2) @binding(108) var terrain_layer2_sampler: sampler;
@group(2) @binding(109) var terrain_layer3_texture: texture_2d<f32>;
@group(2) @binding(110) var terrain_layer3_sampler: sampler;

fn terrain_weights(map_uv: vec2<f32>) -> vec4<f32> {
    let weights = textureSample(terrain_splat_texture, terrain_splat_sampler, map_uv);
    let sum = max(dot(weights, vec4(1.0)), EPSILON);
    return weights / sum;
}

fn terrain_color(uv: vec2<f32>, weights: vec4<f32>) -> vec4<f32> {
    let layer0 = textureSample(terrain_layer0_texture, terrain_layer0_sampler, uv);
    let layer1 = textureSample(terrain_layer1_texture, terrain_layer1_sampler, uv);
    let layer2 = textureSample(terrain_layer2_texture, terrain_layer2_sampler, uv);
    let layer3 = textureSample(terrain_layer3_texture, terrain_layer3_sampler, uv);

    let blended = layer0 * weights.x
        + layer1 * weights.y
        + layer2 * weights.z
        + layer3 * weights.w;

    return vec4(blended.rgb, 1.0);
}

@fragment
fn fragment(
#ifdef MESHLET_MESH_MATERIAL_PASS
    @builtin(position) frag_coord: vec4<f32>,
#else
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
#endif
) -> FragmentOutput {
#ifdef MESHLET_MESH_MATERIAL_PASS
    let in = resolve_vertex_output(frag_coord);
    let is_front = true;
#endif

    var pbr_input = pbr_input_from_standard_material(in, is_front);

    let world_pos = in.world_position.xyz;
    let map_dimensions = max(terrain_settings.map_size, vec2(EPSILON, EPSILON));
    let map_uv = clamp(world_pos.xz / map_dimensions, vec2(0.0), vec2(1.0));
    let weights = terrain_weights(map_uv);

#ifdef VERTEX_UVS
    let scaled_uv = in.uv / terrain_settings.uv_scale;
#else
    let scaled_uv = world_pos.xz / terrain_settings.uv_scale;
#endif

    let base_color = terrain_color(scaled_uv, weights);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, base_color);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}
