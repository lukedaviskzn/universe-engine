//!include("../include/fullscreen_quad.wgsl")

@group(0) @binding(0)
var texture0: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler0: sampler;
@group(0) @binding(2)
var texture1: texture_2d<f32>;
@group(0) @binding(3)
var tex_sampler1: sampler;
@group(0) @binding(4)
var texture2: texture_2d<f32>;
@group(0) @binding(5)
var tex_sampler2: sampler;
@group(0) @binding(6)
var texture3: texture_2d<f32>;
@group(0) @binding(7)
var tex_sampler3: sampler;
@group(0) @binding(8)
var texture4: texture_2d<f32>;
@group(0) @binding(9)
var tex_sampler4: sampler;
@group(0) @binding(10)
var texture5: texture_2d<f32>;
@group(0) @binding(11)
var tex_sampler5: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let colour0 = textureSample(texture0, tex_sampler0, in.tex_coords).rgb;
    let colour1 = textureSample(texture1, tex_sampler1, in.tex_coords).rgb;
    let colour2 = textureSample(texture2, tex_sampler2, in.tex_coords).rgb;
    let colour3 = textureSample(texture3, tex_sampler3, in.tex_coords).rgb;
    let colour4 = textureSample(texture4, tex_sampler4, in.tex_coords).rgb;
    let colour5 = textureSample(texture5, tex_sampler5, in.tex_coords).rgb;

    let colour = colour0 + colour1 + colour2 + colour3 + colour4 + colour5;

    return vec4<f32>(colour, 1.0);
}
