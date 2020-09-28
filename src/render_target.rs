pub struct Target<'a> {
    swapchain: Option<wgpu::SwapChainTexture>,
    texture_view: Option<&'a wgpu::TextureView>,
}

impl<'a> Target<'a> {
    fn from_swapchain(swapchain: wgpu::SwapChainTexture) -> Self {
        Self {
            swapchain: Some(swapchain),
            texture_view: None,
        }
    }
    fn from_view(texture_view: &'a wgpu::TextureView) -> Self {
        Self {
            swapchain: None,
            texture_view: Some(texture_view),
        }
    }
    pub fn view(&self) -> &wgpu::TextureView {
        if let Some(sw) = self.swapchain.as_ref() {
            &sw.view
        } else {
            self.texture_view.unwrap()
        }
    }
}

pub trait RenderTarget {
    fn compatible_surface(&self) -> Option<&wgpu::Surface>;
    fn create(&mut self, device: &wgpu::Device, size: (u32, u32));
    fn format(&self) -> wgpu::TextureFormat;
    fn output(&mut self) -> Target;
    fn on_render(&self, encoder: &mut wgpu::CommandEncoder);
}

pub struct SwapchainTarget {
    surface: wgpu::Surface,
    sc_desc: Option<wgpu::SwapChainDescriptor>,
    swap_chain: Option<wgpu::SwapChain>,
    format: wgpu::TextureFormat,
}

impl SwapchainTarget {
    pub fn new(surface: wgpu::Surface, format: wgpu::TextureFormat) -> Self {
        Self {
            surface,
            sc_desc: None,
            swap_chain: None,
            format,
        }
    }
}

impl RenderTarget for SwapchainTarget {
    fn compatible_surface(&self) -> Option<&wgpu::Surface> {
        Some(&self.surface)
    }

    fn create(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        self.sc_desc = Some(wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: self.format,
            width: size.0,
            height: size.1,
            present_mode: wgpu::PresentMode::Fifo,
        });

        self.swap_chain =
            Some(device.create_swap_chain(&self.surface, self.sc_desc.as_ref().unwrap()));
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.sc_desc.as_ref().unwrap().format
    }

    fn output(&mut self) -> Target {
        let sw = self.swap_chain.as_mut().unwrap();
        //let t = sw.get_current_frame().unwrap();
        let t = sw.get_next_frame().unwrap();
        Target::from_swapchain(t.output)
    }

    fn on_render(&self, _encoder: &mut wgpu::CommandEncoder) {
        // Nothing to do for swapchains.
    }
}

pub struct TextureTarget {
    texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,
    size: Option<(u32, u32)>,
    output_buffer: Option<wgpu::Buffer>,
}

impl TextureTarget {
    pub fn new() -> Self {
        Self {
            texture: None,
            texture_view: None,
            size: None,
            output_buffer: None,
        }
    }

    // pub async fn get_buffer(&self, device: &wgpu::Device) -> Vec<u8> {
    //     let output_buffer = self.output_buffer.as_ref().unwrap();
    //     let out;
    //     {
    //         let slice = output_buffer.slice(..);
    //         let map = slice.map_async(wgpu::MapMode::Read);
    //         // Wait for the buffer to be mapped.
    //         device.poll(wgpu::Maintain::Wait);
    //         let _ = map.await.unwrap(); // Wait for the future to be ready
    //         let view = slice.get_mapped_range();
    //         out = Vec::from(&*view);
    //     }
    //     output_buffer.unmap();
    //     out
    // }
}

impl RenderTarget for TextureTarget {
    fn compatible_surface(&self) -> Option<&wgpu::Surface> {
        None
    }

    fn create(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        self.size = Some(size);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format(),
            usage: wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: Some("RenderTargetTexture"),
        });
        //let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Foo"),
            format: self.format(),
            dimension: wgpu::TextureViewDimension::D2,
            aspect: wgpu::TextureAspect::default(),
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            array_layer_count: 1,
        }
        );
        self.texture = Some(texture);
        self.texture_view = Some(texture_view);

        // Create a buffer that we can use to read data out
        let buffer_size = (size.0 * size.1 * 4) as wgpu::BufferAddress;
        let buffer_desc = wgpu::BufferDescriptor {
            size: buffer_size,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
            label: Some("OutputBuffer"),
            //mapped_at_creation: false,
        };
        self.output_buffer = Some(device.create_buffer(&buffer_desc));
    }

    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }

    fn output(&mut self) -> Target {
        Target::from_view(self.texture_view.as_ref().unwrap())
    }

    fn on_render(&self, encoder: &mut wgpu::CommandEncoder) {
        // Copy the output texture to the mapped buffer.
        let size = self.size.unwrap();
        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: self.texture.as_ref().unwrap(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::BufferCopyView {
                buffer: self.output_buffer.as_ref().unwrap(),
                layout: wgpu::TextureDataLayout {
                    offset: 0 as wgpu::BufferAddress,
                    bytes_per_row: size.0 * 4,
                    rows_per_image: size.1,
                },
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth: 1,
            },
        );
    }
}
