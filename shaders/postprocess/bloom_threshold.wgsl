//!include("../include/fullscreen_quad.wgsl")

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let colour = textureSample(texture, tex_sampler, in.tex_coords);
    // return vec4<f32>(max(colour.rgb - 1.0, vec3<f32>(0.0)) / 4.0, 1.0);
    return colour; // don't threshold
}
