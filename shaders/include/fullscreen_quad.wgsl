
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    var out: VertexOutput;

    let position = vec2<f32>(f32(index / 2) * 2.0 - 1.0, f32((index+1) % 2) * 2.0 - 1.0);
    out.tex_coords = (position + 1.0) / 2.0;
    out.tex_coords.y = 1.0 - out.tex_coords.y;
    out.position = vec4(position, 0.0, 1.0);

    return out;
}
