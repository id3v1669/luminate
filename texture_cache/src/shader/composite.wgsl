@group(0) @binding(0) var cache_texture: texture_2d<f32>;
@group(0) @binding(1) var cache_sampler: sampler;

// Three scalar pads (not a vec3, whose 16-byte alignment would make the
// struct 32 bytes) keep this at exactly 16 bytes, matching the Rust side.
struct Params {
    opacity: f32,
    pad0: f32,
    pad1: f32,
    pad2: f32,
}
@group(0) @binding(2) var<uniform> params: Params;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    // One oversized triangle, (-1,-1) (3,-1) (-1,3), covers the whole
    // viewport after clipping; the visible square maps to uv 0..1.
    let x = f32(i32(index & 1u) * 4 - 1);
    let y = f32(i32(index >> 1u) * 4 - 1);
    var out: VertexOutput;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // The texture holds premultiplied colour, so scaling every channel by the
    // group opacity keeps it premultiplied.
    return textureSample(cache_texture, cache_sampler, in.uv) * params.opacity;
}
