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

#ifdef TERRAIN_MATERIAL_EXTENSION_BASE_COLOR_ARRAY
@group(2) @binding(101)
var terrain_base_color_array: texture_2d_array<f32>;
@group(2) @binding(102)
var terrain_base_color_sampler: sampler;
#endif

#ifdef TERRAIN_MATERIAL_EXTENSION_NORMAL_ARRAY
@group(2) @binding(103)
var terrain_normal_array: texture_2d_array<f32>;
@group(2) @binding(104)
var terrain_normal_sampler: sampler;
#endif

#ifdef TERRAIN_MATERIAL_EXTENSION_ROUGHNESS_ARRAY
@group(2) @binding(105)
var terrain_roughness_array: texture_2d_array<f32>;
@group(2) @binding(106)
var terrain_roughness_sampler: sampler;
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

#ifdef TERRAIN_MATERIAL_EXTENSION_BASE_COLOR_ARRAY
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

//    let layer_f = f32(layer);
//    let x_tex = textureSample(tex, samp, vec3<f32>(uv_x, layer_f));
//    let y_tex = textureSample(tex, samp, vec3<f32>(uv_y, layer_f));
//    let z_tex = textureSample(tex, samp, vec3<f32>(uv_z, layer_f));

    let x_tex = textureSample(tex, samp, uv_x, layer);
    let y_tex = textureSample(tex, samp, uv_y, layer);
    let z_tex = textureSample(tex, samp, uv_z, layer);

    return x_tex * weights.x + y_tex * weights.y + z_tex * weights.z;
}
#endif

#ifdef TERRAIN_MATERIAL_EXTENSION_NORMAL_ARRAY
fn triplanar_sample_layer_normal(
    tex: texture_2d_array<f32>,
    samp: sampler,
    pos: vec3<f32>,
    norm: vec3<f32>,
    scale: f32,
    layer: i32,
) -> vec3<f32> {
    let n = normalize(norm);
    let weights = abs(n) / (abs(n.x) + abs(n.y) + abs(n.z));

    let uv_x = fract(pos.yz * scale);
    let uv_y = fract(pos.xz * scale);
    let uv_z = fract(pos.xy * scale);

    let sample_x = textureSample(tex, samp, uv_x, layer).xyz * 2.0 - vec3<f32>(1.0);
    let sample_y = textureSample(tex, samp, uv_y, layer).xyz * 2.0 - vec3<f32>(1.0);
    let sample_z = textureSample(tex, samp, uv_z, layer).xyz * 2.0 - vec3<f32>(1.0);

    var sign_x: f32;
    if (n.x >= 0.0) {
        sign_x = 1.0;
    } else {
        sign_x = -1.0;
    }

    var sign_y: f32;
    if (n.y >= 0.0) {
        sign_y = 1.0;
    } else {
        sign_y = -1.0;
    }

    var sign_z: f32;
    if (n.z >= 0.0) {
        sign_z = 1.0;
    } else {
        sign_z = -1.0;
    }

    let world_x = normalize(
        sample_x.x * vec3<f32>(0.0, sign_x, 0.0)
            + sample_x.y * vec3<f32>(0.0, 0.0, 1.0)
            + sample_x.z * vec3<f32>(sign_x, 0.0, 0.0),
    );
    let world_y = normalize(
        sample_y.x * vec3<f32>(sign_y, 0.0, 0.0)
            + sample_y.y * vec3<f32>(0.0, 0.0, 1.0)
            + sample_y.z * vec3<f32>(0.0, sign_y, 0.0),
    );
    let world_z = normalize(
        sample_z.x * vec3<f32>(sign_z, 0.0, 0.0)
            + sample_z.y * vec3<f32>(0.0, sign_z, 0.0)
            + sample_z.z * vec3<f32>(0.0, 0.0, sign_z),
    );

    return normalize(world_x * weights.x + world_y * weights.y + world_z * weights.z);
}
#endif

#ifdef TERRAIN_MATERIAL_EXTENSION_ROUGHNESS_ARRAY
fn triplanar_sample_layer_scalar(
    tex: texture_2d_array<f32>,
    samp: sampler,
    pos: vec3<f32>,
    norm: vec3<f32>,
    scale: f32,
    layer: i32,
) -> f32 {
    let n = normalize(norm);
    let weights = abs(n) / (abs(n.x) + abs(n.y) + abs(n.z));

    let uv_x = fract(pos.yz * scale);
    let uv_y = fract(pos.xz * scale);
    let uv_z = fract(pos.xy * scale);

    let sample_x = textureSample(tex, samp, uv_x, layer).g;
    let sample_y = textureSample(tex, samp, uv_y, layer).g;
    let sample_z = textureSample(tex, samp, uv_z, layer).g;

    return sample_x * weights.x + sample_y * weights.y + sample_z * weights.z;
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

#ifdef TERRAIN_MATERIAL_EXTENSION_BASE_COLOR_ARRAY
    if (terrain_material_extension.layer_count > 0u) {
        let max_layer = i32(terrain_material_extension.layer_count) - 1;
#ifdef VERTEX_UVS_B
        let layer_source = in.uv_b.x;


#else
        let layer_source = 0.0;
#endif
        let layer_value = clamp(i32(round(layer_source)), 0, max_layer);
        var sampled = triplanar_sample_layer(
            terrain_base_color_array,
            terrain_base_color_sampler,
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

#ifdef TERRAIN_MATERIAL_EXTENSION_NORMAL_ARRAY
    if (terrain_material_extension.layer_count > 0u) {
        let max_layer = i32(terrain_material_extension.layer_count) - 1;
#ifdef VERTEX_UVS_B
        let layer_source = in.uv_b.x;
#else
        let layer_source = 0.0;
#endif
        let layer_value = clamp(i32(round(layer_source)), 0, max_layer);
        let world_normal = triplanar_sample_layer_normal(
            terrain_normal_array,
            terrain_normal_sampler,
            pbr_input.world_position.xyz,
            pbr_input.world_normal.xyz,
            scale,
            layer_value,
        );
        pbr_input.N = world_normal;
        pbr_input.clearcoat_N = world_normal;
    }


#endif

#ifdef TERRAIN_MATERIAL_EXTENSION_ROUGHNESS_ARRAY
    if (terrain_material_extension.layer_count > 0u) {
        let max_layer = i32(terrain_material_extension.layer_count) - 1;
        #ifdef VERTEX_UVS_B
            let layer_source = in.uv_b.x;
        #else
            let layer_source = 0.0;
        #endif
        let layer_value = clamp(i32(round(layer_source)), 0, max_layer);
        let sampled = triplanar_sample_layer_scalar(
            terrain_roughness_array,
            terrain_roughness_sampler,
            pbr_input.world_position.xyz,
            pbr_input.world_normal.xyz,
            scale,
            layer_value,
        );

        // Remap sampled 0..1 → custom min..max range
        let rough_min: f32 = 0.2;   // tweak this
        let rough_max: f32 = 0.9;   // tweak this
        let rough = clamp(sampled, 0.0, 1.0);
        let remapped = mix(rough_min, rough_max, rough);

        pbr_input.material.perceptual_roughness = clamp(remapped, 0.045, 1.0);
    }
#endif



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

//    #ifdef DEBUG_NORMALSa
//        out.color = vec4<f32>(
//            0.5 * (pbr_input.N.x + 1.0),
//            0.5 * (pbr_input.N.y + 1.0),
//            0.5 * (pbr_input.N.z + 1.0),
//            1.0
//        );
//    #endif

//    out.color = vec4<f32>(in.uv_b.x / 10.0, in.uv_b.y, 0.0, 1.0);


#ifdef DEBUG_ROUGHNESS
if (terrain_material_extension.layer_count > 0u) {
    let max_layer = i32(terrain_material_extension.layer_count) - 1;
    #ifdef VERTEX_UVS_B
    let layer_source = in.uv_b.x;
    #else
    let layer_source = 0.0;
    #endif
    let layer_value = clamp(i32(round(layer_source)), 0, max_layer);

//<<<<<<< HEAD


//let texVal = textureSample(
//    terrain_roughness_array,
//    terrain_roughness_sampler,
//    fract(in.uv_b.xy),
//    layer_value,
//);
//
//out.color = vec4<f32>(texVal.r, texVal.g, texVal.b, 1.0);

//            let tex = textureSample(
//                terrain_roughness_array,
//                terrain_roughness_sampler,
//                fract(in.uv_b.xy),
//                layer_value,
//            );

            // Show channels separately
//out.color = vec4<f32>(tex.g, tex.g, tex.g, 1.0);

//        }
//=======
    // sample via triplanar (same as the real path)
    let sampled = triplanar_sample_layer_scalar(
        terrain_roughness_array,
        terrain_roughness_sampler,
        pbr_input.world_position.xyz,
        pbr_input.world_normal.xyz,
        terrain_material_extension.uv_scale,
        layer_value,
    );

    // make variation visible
    let boosted = clamp((sampled - 0.05) * 6.0, 0.0, 1.0);
    out.color = vec4<f32>(boosted, boosted, boosted, 1.0);
//    return out; // avoid being overwritten by lighting
}
//>>>>>>> e74c3be (Finally get the image to show)
#endif




#endif'

    return out;
}