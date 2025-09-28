#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
};

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
};
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
};
#endif

struct TerrainBlendParams {
    uv_scale: f32,
    inv_map_width: f32,
    inv_map_height: f32,
    _padding: f32,
    layer_tints: array<vec4<f32>, 4>,
}

@group(2) @binding(100)
var<uniform> terrain_params: TerrainBlendParams;

@group(2) @binding(101)
var terrain_splatmap: texture_2d<f32>;
@group(2) @binding(102)
var terrain_splatmap_sampler: sampler;

@group(2) @binding(103)
var terrain_layer0: texture_2d<f32>;
@group(2) @binding(104)
var terrain_layer1: texture_2d<f32>;
@group(2) @binding(105)
var terrain_layer2: texture_2d<f32>;
@group(2) @binding(106)
var terrain_layer3: texture_2d<f32>;
@group(2) @binding(107)
var terrain_layer_sampler: sampler;

fn splat_weights(world_position: vec3<f32>) -> vec4<f32> {
    let splat_uv = vec2<f32>(
        world_position.x * terrain_params.inv_map_width,
        world_position.z * terrain_params.inv_map_height,
    );
    var weights = textureSample(terrain_splatmap, terrain_splatmap_sampler, splat_uv);
    let weight_sum = max(dot(weights, vec4<f32>(1.0)), 1e-4);
    return weights / weight_sum;
}

fn blended_color(weights: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    var color = vec4<f32>(0.0);
    color += textureSample(terrain_layer0, terrain_layer_sampler, uv)
        * terrain_params.layer_tints[0]
        * weights.x;
    color += textureSample(terrain_layer1, terrain_layer_sampler, uv)
        * terrain_params.layer_tints[1]
        * weights.y;
    color += textureSample(terrain_layer2, terrain_layer_sampler, uv)
        * terrain_params.layer_tints[2]
        * weights.z;
    color += textureSample(terrain_layer3, terrain_layer_sampler, uv)
        * terrain_params.layer_tints[3]
        * weights.w;
    return color;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    let weights = splat_weights(in.world_position.xyz);
    let scale = max(terrain_params.uv_scale, 1e-4);
    let tiled_uv = in.uv / scale;
    let surface_color = blended_color(weights, tiled_uv);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, surface_color);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}
