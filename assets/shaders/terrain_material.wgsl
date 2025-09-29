#import bevy_pbr::{
    pbr_functions::{
        alpha_discard,
        apply_normal_mapping,
        apply_pbr_lighting,
        calculate_tbn_mikktspace,
        main_pass_post_lighting_processing,
        sample_texture,
        SampleBias,
    },
    pbr_fragment::pbr_input_from_standard_material,
    pbr_bindings,
    pbr_types,
    mesh_view_bindings::view,
};

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
};
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
};
#endif

#ifdef MESHLET_MESH_MATERIAL_PASS
#import bevy_pbr::meshlet_visibility_buffer_resolve::resolve_vertex_output;
#endif

struct TerrainMaterialExtension {
    uv_scale: f32,
}

@group(2) @binding(100)
var<uniform> terrain_material_extension: TerrainMaterialExtension;

fn terrain_world_uv(world_position: vec4<f32>) -> vec2<f32> {
    return world_position.xz * terrain_material_extension.uv_scale;
}

fn terrain_sample_bias(world_uv: vec2<f32>) -> SampleBias {
    var bias: SampleBias;
#ifdef MESHLET_MESH_MATERIAL_PASS
    bias.ddx_uv = dpdx(world_uv);
    bias.ddy_uv = dpdy(world_uv);
#else
    let _ = world_uv;
    bias.mip_bias = view.mip_bias;
#endif
    return bias;
}

fn terrain_resample_textures(
    in: VertexOutput,
    is_front: bool,
    mut pbr_input: pbr_types::PbrInput,
) -> pbr_types::PbrInput {
    let world_uv = terrain_world_uv(in.world_position);
    let bias = terrain_sample_bias(world_uv);
    let flags = pbr_bindings::material.flags;

    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u {
        var sampled = sample_texture(
            pbr_bindings::base_color_texture,
            pbr_bindings::base_color_sampler,
            world_uv,
            bias,
        );
        sampled *= pbr_bindings::material.base_color;
#ifdef ALPHA_TO_COVERAGE
        let alpha_mode = flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
        if alpha_mode == pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_ALPHA_TO_COVERAGE {
            sampled.a = (sampled.a - pbr_bindings::material.alpha_cutoff) /
                max(fwidth(sampled.a), 0.0001) + 0.5;
        }
#endif
        pbr_input.material.base_color = sampled;
    }

    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT) != 0u {
        let emissive = sample_texture(
            pbr_bindings::emissive_texture,
            pbr_bindings::emissive_sampler,
            world_uv,
            bias,
        );
        pbr_input.material.emissive = vec4<f32>(
            emissive.rgb * pbr_bindings::material.emissive.rgb,
            pbr_bindings::material.emissive.a,
        );
    }

    pbr_input.material.metallic = pbr_bindings::material.metallic;
    pbr_input.material.perceptual_roughness = pbr_bindings::material.perceptual_roughness;
    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT) != 0u {
        let metallic_roughness = sample_texture(
            pbr_bindings::metallic_roughness_texture,
            pbr_bindings::metallic_roughness_sampler,
            world_uv,
            bias,
        );
        pbr_input.material.metallic *= metallic_roughness.b;
        pbr_input.material.perceptual_roughness *= metallic_roughness.g;
    }

    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT) != 0u {
        let occlusion = sample_texture(
            pbr_bindings::occlusion_texture,
            pbr_bindings::occlusion_sampler,
            world_uv,
            bias,
        );
        pbr_input.diffuse_occlusion *= occlusion.rrr;
    }

#ifndef LOAD_PREPASS_NORMALS
    pbr_input.N = normalize(pbr_input.world_normal);
    pbr_input.clearcoat_N = pbr_input.N;

#ifdef VERTEX_TANGENTS
#ifdef STANDARD_MATERIAL_NORMAL_MAP
    let TBN = calculate_tbn_mikktspace(pbr_input.world_normal, in.world_tangent);
    let Nt = sample_texture(
        pbr_bindings::normal_map_texture,
        pbr_bindings::normal_map_sampler,
        world_uv,
        bias,
    ).rgb;
    let double_sided = (flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;
    pbr_input.N = apply_normal_mapping(flags, TBN, double_sided, is_front, Nt);
#endif
#endif
#endif

    return pbr_input;
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
    pbr_input = terrain_resample_textures(in, is_front, pbr_input);

    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

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
