struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.uv = uv;
    return output;
}

@group(0) @binding(0) var y_plane: texture_2d<f32>;
@group(0) @binding(1) var uv_plane: texture_2d<f32>;
@group(0) @binding(2) var sampler_linear: sampler;

// BT.601 coefficients
// https://github.com/spurious/SDL-mirror/blob/4ddd4c445aa059bb127e101b74a8c5b59257fbe2/src/render/opengl/SDL_shaders_gl.c#L93
const offset = vec3<f32>(-0.0627451017, -0.501960814, -0.501960814);
const coeff = mat3x3<f32>(
    1.1644,  0.0000,  1.5960,
    1.1644, -0.3918, -0.8130,
    1.1644,  2.0172,  0.0000,
);

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var yuv: vec3<f32>;
    var rgb: vec3<f32>;

    yuv.x = textureSample(y_plane, sampler_linear, input.uv).r;
    let uv = textureSample(uv_plane, sampler_linear, input.uv).rg;
    yuv.y = uv.r;
    yuv.z = uv.g;

    rgb = (yuv + offset) * coeff;

    return vec4<f32>(rgb, 1.0);
}

fn linear_to_srgb(color: vec3<f32>) -> vec3<f32> {
    return pow(color, vec3(1.0 * 2.2));
}
