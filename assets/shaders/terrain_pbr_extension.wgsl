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

struct TerrainMaterialParams {
    uv_scale: f32,
    tile_type_override: i32,
    layer_count: u32,
    _pad: u32,
}

@group(2) @binding(100)
var<uniform> terrain_material_params: TerrainMaterialParams;
@group(2) @binding(101)
var terrain_base_color_array: texture_2d_array<f32>;

fn triplanar_sample_array(
    layer: i32,
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

    let layer_f = f32(layer);
    let x_tex = textureSample(terrain_base_color_array, pbr_bindings::base_color_sampler, vec3<f32>(uv_x, layer_f));
    let y_tex = textureSample(terrain_base_color_array, pbr_bindings::base_color_sampler, vec3<f32>(uv_y, layer_f));
    let z_tex = textureSample(terrain_base_color_array, pbr_bindings::base_color_sampler, vec3<f32>(uv_z, layer_f));

    return x_tex * weights.x + y_tex * weights.y + z_tex * weights.z;
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

#ifdef VISIBILITY_RANGE_DITHER
    pbr_functions::visibility_range_dither(in.position, in.visibility_range_dither);
#endif

    // Build PBR input (uses mesh UVs initially)
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // Choose projection by dominant world normal axis
    let an = abs(pbr_input.N); // vec3<f32>
    if (terrain_material_params.layer_count > 0u) {
        var desired_layer = terrain_material_params.tile_type_override;
        if (desired_layer < 0) {
            desired_layer = i32(round(in.uv1.x));
        }
        let max_layer = i32(terrain_material_params.layer_count) - 1;
        let clamped_layer = clamp(desired_layer, 0, max_layer);

        let world_base = triplanar_sample_array(
            clamped_layer,
            pbr_input.world_position.xyz,
            pbr_input.world_normal.xyz,
            terrain_material_params.uv_scale,
        );
        pbr_input.material.base_color = alpha_discard(pbr_input.material, world_base);
    } else {
        pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    }


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