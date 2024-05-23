struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) colour: vec3<f32>,
};

struct InstanceInput {
    @location(2) model0: vec4<f32>,
    @location(3) model1: vec4<f32>,
    @location(4) model2: vec4<f32>,
    @location(5) model3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) colour: vec3<f32>,
    @location(1) position: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> vp: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> fovy_factor: f32;

@group(2) @binding(0)
var<uniform> model: mat4x4<f32>;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    // let model = mat4x4<f32>(
    //     instance.model0,
    //     instance.model1,
    //     instance.model2,
    //     instance.model3,
    // );

    let world_pos = model * vec4<f32>(vertex.position, 1.0);

    var out: VertexOutput;
    out.colour = vertex.colour;
    out.position = world_pos.xyz;
    out.clip_position = vp * world_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let attenuation = 1.0 + length(in.position / 1.0e8) * 1.0e8;
    let att = attenuation / 1.0e16; // scale to prevent overflow
    return vec4<f32>(in.colour / 1.0e24 / att / att * fovy_factor, 1.0); // in.colour / (1.0e16)^2 * 1.0e8
}
