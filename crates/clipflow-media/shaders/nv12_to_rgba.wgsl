struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(0) var sampler_linear: sampler;
@group(0) @binding(1) var texture_y: texture_2d<f32>;
@group(0) @binding(2) var texture_uv: texture_2d<f32>;

struct ColorParams {
    matrix_row0: vec4<f32>,
    matrix_row1: vec4<f32>,
    matrix_row2: vec4<f32>,
    yuv_offset: vec4<f32>,
};
@group(0) @binding(3) var<uniform> params: ColorParams;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // 全屏大三角形算法 (无需顶点缓冲)
    let x = f32(i32(in_vertex_index & 1u) * 4 - 1);
    let y = f32(i32(in_vertex_index & 2u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 采样 Y 单通道与 UV 双通道 (硬件双线性插值自动完成 UV 2x 上采样)
    let y = textureSample(texture_y, sampler_linear, in.tex_coords).r;
    let uv = textureSample(texture_uv, sampler_linear, in.tex_coords).rg;

    // 扣除 YUV 动态偏置 (Limited Range 下 Y-0.062745, UV-0.50196)
    let yuv = vec3<f32>(y, uv.x, uv.y) - params.yuv_offset.xyz;

    // 无分支矩阵点乘快速转为标准 RGB
    let r = dot(params.matrix_row0.xyz, yuv);
    let g = dot(params.matrix_row1.xyz, yuv);
    let b = dot(params.matrix_row2.xyz, yuv);

    return vec4<f32>(clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
