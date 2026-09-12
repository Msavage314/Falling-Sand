use crate::{config, fullscreen_pass::FullscreenPass};
pub struct BloomEffect {
    sampler: pixels::wgpu::Sampler,
    source_texture: pixels::wgpu::Texture,
    source_view: pixels::wgpu::TextureView,
    blur_a_view: pixels::wgpu::TextureView,
    blur_b_view: pixels::wgpu::TextureView,
    blur_pass: FullscreenPass,
    horizontal_group: pixels::wgpu::BindGroup,
    vertical_group: pixels::wgpu::BindGroup,
    composite_pass: FullscreenPass,
    composite_group: pixels::wgpu::BindGroup,
}

impl BloomEffect {
    pub fn new(device: &pixels::wgpu::Device, surface_format: pixels::wgpu::TextureFormat) -> Self {
        let width = config::WIDTH as u32;
        let height = config::HEIGHT as u32;
        let format = pixels::wgpu::TextureFormat::Rgba8UnormSrgb;

        let make_texture = |label: &str| {
            device.create_texture(&pixels::wgpu::TextureDescriptor {
                label: Some(label),
                size: pixels::wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: pixels::wgpu::TextureDimension::D2,
                format,
                usage: pixels::wgpu::TextureUsages::TEXTURE_BINDING
                    | pixels::wgpu::TextureUsages::COPY_DST
                    | pixels::wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        };

        let source_texture = make_texture("bloom-source");
        let source_view =
            source_texture.create_view(&pixels::wgpu::TextureViewDescriptor::default());
        let blur_a_texture = make_texture("bloom-blur-a");
        let blur_a_view =
            blur_a_texture.create_view(&pixels::wgpu::TextureViewDescriptor::default());
        let blur_b_texture = make_texture("bloom-blur-b");
        let blur_b_view =
            blur_b_texture.create_view(&pixels::wgpu::TextureViewDescriptor::default());
        let additive_blend = pixels::wgpu::BlendState {
            color: pixels::wgpu::BlendComponent {
                src_factor: pixels::wgpu::BlendFactor::One,
                dst_factor: pixels::wgpu::BlendFactor::One,
                operation: pixels::wgpu::BlendOperation::Add,
            },
            alpha: pixels::wgpu::BlendComponent::REPLACE,
        };
        let sampler = device.create_sampler(&pixels::wgpu::SamplerDescriptor {
            address_mode_u: pixels::wgpu::AddressMode::ClampToEdge,
            address_mode_v: pixels::wgpu::AddressMode::ClampToEdge,
            mag_filter: pixels::wgpu::FilterMode::Linear,
            min_filter: pixels::wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let texel_x = 1.0 / width as f32;
        let texel_y = 1.0 / height as f32;
        let make_params_buffer = |dir_texel: [f32; 4]| {
            use pixels::wgpu::util::DeviceExt;
            device.create_buffer_init(&pixels::wgpu::util::BufferInitDescriptor {
                label: Some("blur-params"),
                contents: bytemuck::cast_slice(&dir_texel),
                usage: pixels::wgpu::BufferUsages::UNIFORM,
            })
        };
        let h_params = make_params_buffer([texel_x, 0.0, 0.0, 0.0]);
        let v_params = make_params_buffer([0.0, texel_y, 0.0, 0.0]);
        let blur_pass = FullscreenPass::new(
            device,
            "blur",
            include_str!("../shaders/blur.wgsl"),
            pixels::wgpu::TextureFormat::Rgba8UnormSrgb,
            pixels::wgpu::BlendState::REPLACE,
            Some(16),
        );
        let horizontal_group =
            blur_pass.bind_group(device, &source_view, &sampler, Some(&h_params));
        let vertical_group = blur_pass.bind_group(device, &blur_a_view, &sampler, Some(&v_params));

        let composite_pass = FullscreenPass::new(
            device,
            "composite",
            include_str!("../shaders/compsite.wgsl"),
            surface_format,
            additive_blend,
            None,
        );
        let composite_group = composite_pass.bind_group(device, &blur_b_view, &sampler, None);

        Self {
            blur_a_view,
            blur_b_view,
            sampler,
            source_texture,
            source_view,
            blur_pass,
            horizontal_group,
            vertical_group,
            composite_pass,
            composite_group,
        }
    }

    pub fn render(
        &self,
        encoder: &mut pixels::wgpu::CommandEncoder,
        render_target: &pixels::wgpu::TextureView,
        viewport: (f32, f32, f32, f32),
    ) {
        self.blur_pass.run(
            encoder,
            &self.blur_a_view,
            &self.horizontal_group,
            true,
            None,
        );
        self.blur_pass
            .run(encoder, &self.blur_b_view, &self.vertical_group, true, None);
        self.composite_pass.run(
            encoder,
            render_target,
            &self.composite_group,
            false,
            Some(viewport),
        );
    }
    /// Upload the freshly-drawn bloom mask (RGBA8, WIDTH*HEIGHT*4 bytes) to the GPU.
    pub fn upload(&self, queue: &pixels::wgpu::Queue, bloom_buffer: &[u8]) {
        queue.write_texture(
            pixels::wgpu::TexelCopyTextureInfo {
                texture: &self.source_texture,
                mip_level: 0,
                origin: pixels::wgpu::Origin3d::ZERO,
                aspect: pixels::wgpu::TextureAspect::All,
            },
            bloom_buffer,
            pixels::wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * config::WIDTH as u32),
                rows_per_image: Some(config::HEIGHT as u32),
            },
            pixels::wgpu::Extent3d {
                width: config::WIDTH as u32,
                height: config::HEIGHT as u32,
                depth_or_array_layers: 1,
            },
        );
    }
}
