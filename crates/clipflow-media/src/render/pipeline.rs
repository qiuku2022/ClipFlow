pub const NV12_WGSL_SHADER: &str = include_str!("../../shaders/nv12_to_rgba.wgsl");

pub struct Nv12RenderPipeline;

impl Nv12RenderPipeline {
    pub fn shader_source() -> &'static str {
        NV12_WGSL_SHADER
    }

    /// 在给定 wgpu::Device 上创建着色器模块
    pub fn create_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("NV12 to RGBA Shader"),
            source: wgpu::ShaderSource::Wgsl(NV12_WGSL_SHADER.into()),
        })
    }
}
