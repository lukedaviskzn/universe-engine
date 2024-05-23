//!include("../include/fullscreen_quad.wgsl")

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let rc = 1.0025;
    let gc = 1.0;
    let bc = 0.9975;
    
    let r_tc = in.tex_coords*rc - vec2(0.5)*(rc - 1.0);
    let g_tc = in.tex_coords*gc - vec2(0.5)*(gc - 1.0);
    let b_tc = in.tex_coords*bc - vec2(0.5)*(bc - 1.0);

    let r = textureSample(texture, tex_sampler, r_tc).r;
    let g = textureSample(texture, tex_sampler, g_tc).g;
    let b = textureSample(texture, tex_sampler, b_tc).b;
    
    return vec4(r, g, b, 1.0);
}
