#import bevy_pbr::{
    pbr_bindings,
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
};

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
};
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
};
#endif

#ifdef MESHLET_MESH_MATERIAL_PASS
#import bevy_pbr::meshlet_visibility_buffer_resolve::resolve_vertex_output
#endif

struct TerrainMaterialExtension {
    uv_scale: f32,
    layer_count: u32,
    _padding: vec2<f32>,
}

@group(2) @binding(100)
var<uniform> terrain_material_extension: TerrainMaterialExtension;

#ifdef TERRAIN_MATERIAL_EXTENSION_TEXTURE_ARRAY
@group(2) @binding(101)
var terrain_texture_array: texture_2d_array<f32>;
@group(2) @binding(102)
var terrain_texture_sampler: sampler;
#endif

fn triplanar_sample(
    tex: texture_2d<f32>,
    samp: sampler,
    pos: vec3<f32>,
    norm: vec3<f32>,
    scale: f32
) -> vec4<f32> {
    let n = normalize(norm);
    let weights = abs(n) / (abs(n.x) + abs(n.y) + abs(n.z));

    // Wrap each projection into [0,1)
    let uv_x = fract(pos.yz * scale);
    let uv_y = fract(pos.xz * scale);
    let uv_z = fract(pos.xy * scale);

    let x_tex = textureSample(tex, samp, uv_x);
    let y_tex = textureSample(tex, samp, uv_y);
    let z_tex = textureSample(tex, samp, uv_z);

    return x_tex * weights.x + y_tex * weights.y + z_tex * weights.z;
}

#ifdef TERRAIN_MATERIAL_EXTENSION_TEXTURE_ARRAY
fn triplanar_sample_layer(
    tex: texture_2d_array<f32>,
    samp: sampler,
    pos: vec3<f32>,
    norm: vec3<f32>,
    scale: f32,
    layer: i32,
) -> vec4<f32> {
    let n = normalize(norm);
    let weights = abs(n) / (abs(n.x) + abs(n.y) + abs(n.z));

    let uv_x = fract(pos.yz * scale);
    let uv_y = fract(pos.xz * scale);
    let uv_z = fract(pos.xy * scale);

    let layer_f = f32(layer);
    let x_tex = textureSample(tex, samp, vec3<f32>(uv_x, layer_f));
    let y_tex = textureSample(tex, samp, vec3<f32>(uv_y, layer_f));
    let z_tex = textureSample(tex, samp, vec3<f32>(uv_z, layer_f));

    return x_tex * weights.x + y_tex * weights.y + z_tex * weights.z;
}
#endif




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

#ifdef VISIBILITY_RANGE_DITHER
    pbr_functions::visibility_range_dither(in.position, in.visibility_range_dither);
#endif

    // Build PBR input (uses mesh UVs initially)
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // Choose projection by dominant world normal axis
    let scale = terrain_material_extension.uv_scale;
    var base_color = vec4<f32>(pbr_input.material.base_color.rgb, 1.0);

#ifdef STANDARD_MATERIAL_BASE_COLOR_TEXTURE
    base_color = triplanar_sample(
        pbr_bindings::base_color_texture,
        pbr_bindings::base_color_sampler,
        pbr_input.world_position.xyz,
        pbr_input.world_normal.xyz,
        scale,
    );
#endif

#ifdef TERRAIN_MATERIAL_EXTENSION_TEXTURE_ARRAY
    if (terrain_material_extension.layer_count > 0u) {
        let max_layer = i32(terrain_material_extension.layer_count) - 1;
        let layer_value = clamp(i32(round(in.uv2.x)), 0, max_layer);
        var sampled = triplanar_sample_layer(
            terrain_texture_array,
            terrain_texture_sampler,
            pbr_input.world_position.xyz,
            pbr_input.world_normal.xyz,
            scale,
            layer_value,
        );
        sampled.a = 1.0;
        base_color = sampled;
    }
#endif

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