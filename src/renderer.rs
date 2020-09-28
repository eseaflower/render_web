use crate::{
    render_target::{RenderTarget, TextureTarget},
    vertex::{Quad, Vertex},
    view_state::ViewState,
};
use std::io::prelude::*;
use std::{mem, path::Path};
//use wgpu::util::DeviceExt;

pub struct State<T>
where
    T: RenderTarget,
{
    target: T,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: (u32, u32),
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    texture_sampler: wgpu::Sampler,
    texture_bind_group: wgpu::BindGroup,
    current_image_no: u8,
    quad: Quad,
    dirty: bool,
    view: ViewState,
    // brush: wgpu_glyph::GlyphBrush<()>,
    // belt: wgpu::util::StagingBelt,
    status_message: Option<String>,
}

impl<T> State<T>
where
    T: RenderTarget,
{
    pub async fn new(instance: wgpu::Instance, size: (u32, u32), mut target: T) -> Self {
        let adapter = instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::Default,
                    compatible_surface: target.compatible_surface(),
                },
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .unwrap();

        log::info!("Adapter created");

        // let (device, queue) = adapter
        //     .request_device(
        //         &wgpu::DeviceDescriptor {
        //             features: wgpu::Features::default(),
        //             limits: wgpu::Limits::default(),
        //             shader_validation: true,
        //         },
        //         None,
        //     )
        //     .await
        //     .expect("Failed to create device");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    limits: wgpu::Limits::default(),
                    extensions: wgpu::Extensions::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        log::info!("Device/Queue created");
        // Create the render target
        target.create(&device, size);

        log::info!("RenderTarget created");

        let (texture, texture_view, texture_sampler, texture_bind_group, texture_bind_group_layput) =
            Self::create_texture(&device);

        log::info!("Texture created");

        let render_pipeline =
            Self::build_render_pipeline(&device, target.format(), texture_bind_group_layput);

        log::info!("Pipeline created");

        // Build the "model" we will use
        let quad = Quad::with_init((size.0 as f32, size.1 as f32));
        log::info!("Quad created");
        let (vertex_buffer, index_buffer) = Self::build_vertex_buffer(&device, &queue, &quad);

        log::info!("Quad and buffers created");

        // Build glyph-brush
        // let font = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!(
        //     "../fonts/open-sans/OpenSans-Regular.ttf"
        // ))
        // .expect("Failed to load font");

        // log::info!("FontArc created");

        // let brush = wgpu_glyph::GlyphBrushBuilder::using_font(font).build(&device, target.format());
        // log::info!("GlyphBrush created");
        // let belt = wgpu::util::StagingBelt::new(1024);

        log::info!("Brush and Belt created");

        Self {
            target,
            //surface,
            device,
            queue,
            // sc_desc,
            // swap_chain,
            size,
            clear_color: wgpu::Color::BLACK,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            texture,
            texture_view,
            texture_sampler,
            texture_bind_group,
            current_image_no: 0,
            quad,
            dirty: true,
            view: ViewState::new(),
            // brush,
            // belt,
            status_message: None,
        }
    }

    fn create_texture(
        device: &wgpu::Device,
    ) -> (
        wgpu::Texture,
        wgpu::TextureView,
        wgpu::Sampler,
        wgpu::BindGroup,
        wgpu::BindGroupLayout,
    ) {
        let texture_size = wgpu::Extent3d {
            width: 1024,
            height: 1024,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            //format: wgpu::TextureFormat::Rgba8UnormSrgb,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: Some("MyTexture"),
        });

        // Create a texture view
        //let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            //format: wgpu::TextureFormat::Rgba8UnormSrgb,
            format: wgpu::TextureFormat::Rgba8Unorm,
            dimension: wgpu::TextureViewDimension::D2,
            aspect: wgpu::TextureAspect::default(),
            base_mip_level: 0,
            base_array_layer: 0,
            level_count: 1,
            array_layer_count: 1,
        });
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Undefined, //compare: None,
            anisotropy_clamp: 1,                       //anisotropy_clamp: None,
            mipmap_filter: wgpu::FilterMode::Nearest,
            label: Some("MySampler"),
        });

        // Create a bind group for the texture.
        let texture_bind_group_layput =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("MyBindgroupLayout"),
                //entries: &[
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("MyBindGroup"),
            layout: &texture_bind_group_layput,
            //entries: &[
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        });

        (
            texture,
            texture_view,
            texture_sampler,
            texture_bind_group,
            texture_bind_group_layput,
        )
    }

    fn build_vertex_buffer(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        quad: &Quad,
    ) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vbuf"),
            size: quad.vertex_count() as u64 * mem::size_of::<Vertex>() as wgpu::BufferAddress,
            //mapped_at_creation: false,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        log::info!("Empty buffer created");

        // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("MyIndexBuffer"),
        //     contents: bytemuck::cast_slice(quad.index_ref()),
        //     usage: wgpu::BufferUsage::INDEX,
        // });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("MyIndexBuffer"),
            usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::COPY_DST,
            size: quad.index_count() as u64 * mem::size_of::<u16>() as wgpu::BufferAddress,
        });

        queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(quad.index_ref()));

        log::info!("Init buffer created");
        (vertex_buffer, index_buffer)
    }

    // fn create_shader_from_file(device: &wgpu::Device, filename: &Path) -> wgpu::ShaderModule {
    //     let buffer =
    //         std::fs::read(filename).expect(&format!("Failed to read {}", filename.display()));
    //     let shader_data = wgpu::util::make_spirv(&buffer);
    //     device.create_shader_module(shader_data)
    // }

    // fn compile_shaders(device: &wgpu::Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    //     let vs_module = Self::create_shader_from_file(device, Path::new("vert.spirv"));
    //     let fs_module = Self::create_shader_from_file(device, Path::new("frag.spirv"));

    //     (vs_module, fs_module)
    // }

    fn shaders_from_static(device: &wgpu::Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
        let vs_data = include_bytes!("../vert.spirv");
        //let fs_data = include_bytes!("../frag.spirv");
        let fs_data = include_bytes!("../frag_static.spirv");
        let vs_module = device
            .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs_data[..])).unwrap());
        let fs_module = device
            .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs_data[..])).unwrap());

        // let vs_module = device.create_shader_module(wgpu::include_spirv!("../vert.spirv"));
        // let fs_module = device.create_shader_module(wgpu::include_spirv!("../frag.spirv"));
        (vs_module, fs_module)
    }

    fn build_render_pipeline(
        device: &wgpu::Device,
        swap_texture_format: wgpu::TextureFormat,
        texture_bind_group_layout: wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        // Compile the shaders
        //let (vs_module, fs_module) = Self::compile_shaders(device);

        // Use static shaders (i.e. included in the binary)
        let (vs_module, fs_module) = Self::shaders_from_static(device);

        log::info!("Shaders created");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&texture_bind_group_layout],
                // push_constant_ranges: &[],
                // label: None,
            });

        log::info!("Pipeline Layout created");

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            //label: None,
            //layout: Some(&render_pipeline_layout),
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                //clamp_depth: false,
                depth_bias: 0,
                depth_bias_clamp: 0.0,
                depth_bias_slope_scale: 0.0,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: swap_texture_format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[Vertex::to_desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.size = (new_size.0, new_size.1);
        self.target.create(&self.device, self.size);
        self.quad
            .set_viewport_size((self.size.0 as f32, self.size.1 as f32));

        self.dirty = true;
    }

    fn update_vertex_buffer(&self) {
        self.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.quad.get_vertex(&self.view)),
        );
    }

    pub fn render(&mut self) {

        //log::info!("Render pos: {:?}", self.view.pos);

        // Make sure the vertex buffer is updated before rendering.
        self.update_vertex_buffer();

        let render_target = self.target.output();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: render_target.view(),
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    clear_color: self.clear_color,
                    store_op: wgpu::StoreOp::Store,
                    // ops: wgpu::Operations {
                    //     load: wgpu::LoadOp::Clear(self.clear_color),
                    //     store: true,
                    // },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..));
            render_pass.draw_indexed(0..self.quad.index_count(), 0, 0..1);
        }

        // if let Some(msg) = &self.status_message {
        //     // Text drawing
        //     let section = wgpu_glyph::Section {
        //         screen_position: (10.0, 10.0),
        //         text: vec![wgpu_glyph::Text::new(&msg)
        //             .with_color([1.0, 1.0, 1.0, 1.0])
        //             .with_scale(40.0)],
        //         ..wgpu_glyph::Section::default()
        //     };
        //     self.brush.queue(section);
        //     self.brush
        //         .draw_queued(
        //             &self.device,
        //             &mut self.belt,
        //             &mut encoder,
        //             render_target.view(),
        //             self.size.0,
        //             self.size.1,
        //         )
        //         .expect("Failed to draw text");
        //     self.belt.finish();
        // }

        self.target.on_render(&mut encoder); // Add any additional commands.

        self.queue.submit(std::iter::once(encoder.finish()));

        // Recall unused staging buffers??
        //let r = self.belt.recall();
        //futures::executor::block_on(r);

        self.dirty = false;

        //log::info!("Render");
    }

    pub fn update_position(&mut self, pos: (f32, f32)) {
        self.view.set_position((pos.0, pos.1));
        //log::info!("Update: {:?}", self.view);

        self.dirty = true;
    }

    pub fn update_zoom(&mut self, pos: (f32, f32)) {
        self.view.set_zoom((pos.0, pos.1));

        self.dirty = true;
    }

    pub fn clear_anchor(&mut self) {
        self.view.clear_anchor();
    }

    pub fn swap_image(&mut self) {
        self.current_image_no += 1;
        let fname = format!(r"e:\temp\video_frames\{}.png", self.current_image_no);
        println!("Loading image: {}", fname);
        //let measure = std::time::Instant::now();
        // log::info!("Attempting to load an image....");
        // let new_image = image::open(Path::new(&fname)).unwrap().into_rgba();
        let image_bytes = include_bytes!(r"e:\temp\video_frames\0.png");
        let new_image = image::load_from_memory(image_bytes).unwrap().into_rgba();

        //log::info!("Open took: {} ms", measure.elapsed().as_millis());
        let image_dims = new_image.dimensions();


        // Queue the copy of the texture data
        self.queue.write_texture(
            wgpu::TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &new_image,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * image_dims.0,
                rows_per_image: image_dims.1,
            },
            wgpu::Extent3d {
                width: image_dims.0,
                height: image_dims.1,
                depth: 1,
            },

        );

        self.quad.map_texture_coords(
            (image_dims.0 as f32, image_dims.1 as f32),
            (1024_f32, 1024_f32),
        );
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl State<TextureTarget> {
    pub async fn get_render_target_data(&self) -> Vec<u8> {
        //self.target.get_buffer(&self.device).await
        Vec::new()
    }
}
