//!include("../include/fullscreen_quad.wgsl")

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var colour = textureSample(texture, tex_sampler, in.tex_coords).rgb;
    // reinhardt
    // let colour = colour / (1.0 + colour);
    // ACES (https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/)
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    colour = colour*(a*colour + b)/(colour*(c*colour + d)+e);
    return vec4<f32>(colour, 1.0);
}
