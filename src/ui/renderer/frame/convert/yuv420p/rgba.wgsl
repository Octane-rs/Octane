struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) tex_coord: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4(position, 1.0);
    output.tex_coord = tex_coord;
    return output;
}

@group(0) @binding(0) var y_tex: texture_2d<f32>;
@group(0) @binding(1) var u_tex: texture_2d<f32>;
@group(0) @binding(2) var v_tex: texture_2d<f32>;
@group(0) @binding(3) var yuv_sampler: sampler;

// BT.709 coefficients
// https://github.com/spurious/SDL-mirror/blob/4ddd4c445aa059bb127e101b74a8c5b59257fbe2/src/render/opengl/SDL_shaders_gl.c#L102
const offset = vec3<f32>(-0.0627451017, -0.501960814, -0.501960814);
const r_coeff = vec3(1.1644,  0.0000,  1.7927);
const g_coeff = vec3(1.1644, -0.2132, -0.5329);
const b_coeff = vec3(1.1644,  2.1124,  0.0000);

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
		var yuv: vec3<f32>;
		var rgb: vec3<f32>;

    yuv.x = textureSample(y_tex, yuv_sampler, input.tex_coord).r;
    yuv.y = textureSample(u_tex, yuv_sampler, input.tex_coord).r;
    yuv.z = textureSample(v_tex, yuv_sampler, input.tex_coord).r;

    yuv = yuv + offset;

    rgb.r = dot(yuv, r_coeff);
    rgb.g = dot(yuv, g_coeff);
    rgb.b = dot(yuv, b_coeff);

    return vec4<f32>(rgb, 1.0);
}

fn linear_to_srgb(color: vec3<f32>) -> vec3<f32> {
    return pow(color, vec3(1.0 * 2.2));
}
