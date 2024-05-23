//!include("../include/fullscreen_quad.wgsl")

const SAMPLE_COUNT: i32 = 9;

fn blur(src_tex: texture_2d<f32>, src_sampler: sampler, blur_dir: vec2<f32>, pix_coord: vec2<f32>) -> vec4<f32> {
    var OFFSETS: array<f32, SAMPLE_COUNT> = array(
        -7.385486338269373,
        -5.415332322090894,
        -3.4458098836553415,
        -1.4767017588568079,
        0.492228282731395,
        2.4612181104350137,
        4.4305055426526785,
        6.400317149797591,
        8
    );
    var WEIGHTS: array<f32, SAMPLE_COUNT> = array(
        0.036514415685046854,
        0.0809315020373954,
        0.1404066727610046,
        0.190680554683392,
        0.20271650855234985,
        0.16870974611035225,
        0.1099127158139171,
        0.056052075960067727,
        0.014075808396474473
    );

    var result = vec4<f32>(0.0);
    let size = vec2<f32>(textureDimensions(src_tex, 0));
    for (var i: i32 = 0; i < SAMPLE_COUNT; i++) {
        let offset = blur_dir * OFFSETS[i] / size;
        let weight = WEIGHTS[i];
        result += textureSample(src_tex, src_sampler, pix_coord + offset) * weight;
    }
    return result;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return blur(texture, tex_sampler, vec2<f32>(BLUR_DIR_X, BLUR_DIR_Y), in.tex_coords);
}
