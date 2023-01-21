use crate::logging::LogLevel;
pub struct Context {
    device: wgpu::Device,
    queue: wgpu::Queue
}

impl Context {

    pub async fn new() -> crate::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .ok_or(crate::Error::new("Failed to request wgpu adapter", LogLevel::Error))?;

        let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await?;

        Ok(Self { device, queue })
    }

    /*pub fn new_model(&self, layers: Vec<Box<dyn super::Layer>>) -> super::Model {
        let encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        //encoder.copy_buffer_to_buffer(source, source_offset, destination, destination_offset, copy_size)
        super::Model {
            encoder,
            layers: todo!(),
        }
    }*/
}