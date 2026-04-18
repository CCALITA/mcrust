use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::camera::{Camera, CameraUniform};
use crate::mesh::{ChunkMesh, Vertex};
use crate::shader::{SHADER_SOURCE, SKY_SHADER_SOURCE};
use crate::sky::DayNightCycle;
use crate::texture::TextureAtlas;

/// Core wgpu renderer. Does NOT own the event loop — mc-client drives that.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture_view: wgpu::TextureView,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_atlas: TextureAtlas,
    // Sky rendering
    sky_pipeline: wgpu::RenderPipeline,
    sky_buffer: wgpu::Buffer,
    sky_bind_group: wgpu::BindGroup,
    /// Shared sky bind group used by the terrain pass (bound at group 2).
    terrain_sky_bind_group: wgpu::BindGroup,
    size: (u32, u32),
}

/// Errors that can occur during renderer initialization.
#[derive(Debug)]
pub enum RendererError {
    /// Failed to create the wgpu surface from the window.
    SurfaceCreation(String),
    /// No compatible GPU adapter was found.
    NoAdapter,
    /// Failed to request a logical device from the adapter.
    DeviceCreation(String),
}

impl std::fmt::Display for RendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SurfaceCreation(msg) => write!(f, "failed to create surface: {msg}"),
            Self::NoAdapter => write!(f, "no suitable GPU adapter found"),
            Self::DeviceCreation(msg) => write!(f, "failed to create device: {msg}"),
        }
    }
}

impl std::error::Error for RendererError {}

impl Renderer {
    /// Initialize the renderer using a winit window.
    /// This blocks on async wgpu initialization via pollster.
    pub fn new(window: Arc<winit::window::Window>) -> Result<Self, RendererError> {
        pollster::block_on(Self::new_async(window))
    }

    async fn new_async(window: Arc<winit::window::Window>) -> Result<Self, RendererError> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        log::info!("Initializing wgpu ({width}x{height})...");

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .map_err(|e| RendererError::SurfaceCreation(e.to_string()))?;

        log::info!("Requesting GPU adapter...");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RendererError::NoAdapter)?;

        log::info!("Adapter: {:?}", adapter.get_info().name);
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("mc_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await
            .map_err(|e| RendererError::DeviceCreation(e.to_string()))?;

        log::info!("GPU device created, configuring surface...");
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Texture atlas
        let texture_atlas = TextureAtlas::new(&device, &queue);

        // ── Sky uniform buffer + bind group ─────────────────────────────
        let sky_uniform = DayNightCycle::default().uniform();
        let sky_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sky_buffer"),
            contents: bytemuck::cast_slice(&[sky_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sky_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sky_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let sky_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky_bind_group"),
            layout: &sky_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sky_buffer.as_entire_binding(),
            }],
        });

        // A second bind group pointing to the same buffer, for the terrain pass
        // (bound at group(2) in the terrain shader).
        let terrain_sky_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain_sky_bind_group"),
            layout: &sky_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sky_buffer.as_entire_binding(),
            }],
        });

        // ── Camera uniform buffer + bind group ─────────────────────────
        let camera_uniform = CameraUniform {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // ── Sky pipeline ────────────────────────────────────────────────
        let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sky_shader"),
            source: wgpu::ShaderSource::Wgsl(SKY_SHADER_SOURCE.into()),
        });

        let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky_pipeline_layout"),
            bind_group_layouts: &[&sky_bind_group_layout],
            push_constant_ranges: &[],
        });

        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky_pipeline"),
            layout: Some(&sky_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sky_shader,
                entry_point: Some("vs_sky"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sky_shader,
                entry_point: Some("fs_sky"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            // Sky writes at max depth; terrain is drawn with depth test on top.
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // ── Terrain pipeline ────────────────────────────────────────────
        let terrain_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("voxel_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &texture_atlas.bind_group_layout,
                &sky_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &terrain_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &terrain_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let depth_texture_view = create_depth_texture(&device, width, height);

        log::info!("Renderer initialized successfully");

        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            depth_texture_view,
            camera_buffer,
            camera_bind_group,
            texture_atlas,
            sky_pipeline,
            sky_buffer,
            sky_bind_group,
            terrain_sky_bind_group,
            size: (width, height),
        })
    }

    /// Access the device (needed by ChunkMesh::build).
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Access the queue.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Handle window resize.
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        let width = new_width.max(1);
        let height = new_height.max(1);
        if (width, height) == self.size {
            return;
        }
        self.size = (width, height);
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture_view = create_depth_texture(&self.device, width, height);
    }

    /// Current surface size.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Write the sky uniform data to the GPU buffer.
    pub fn update_sky(&self, sky: &DayNightCycle) {
        let uniform = sky.uniform();
        self.queue
            .write_buffer(&self.sky_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Render a frame with the given camera, sky state, and chunk meshes.
    ///
    /// Rendering order:
    /// 1. Sky full-screen triangle (at max depth)
    /// 2. Terrain chunks (with depth test on top of sky)
    pub fn render_frame(
        &self,
        camera: &Camera,
        sky: &DayNightCycle,
        chunk_meshes: &[ChunkMesh],
    ) -> Result<(), wgpu::SurfaceError> {
        // Update camera uniform
        let uniform = camera.uniform();
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));

        // Update sky uniform
        self.update_sky(sky);

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 1. Draw sky (full-screen triangle at max depth)
            render_pass.set_pipeline(&self.sky_pipeline);
            render_pass.set_bind_group(0, &self.sky_bind_group, &[]);
            render_pass.draw(0..3, 0..1);

            // 2. Draw terrain chunks
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_atlas.bind_group, &[]);
            render_pass.set_bind_group(2, &self.terrain_sky_bind_group, &[]);

            for mesh in chunk_meshes {
                if mesh.index_count == 0 {
                    continue;
                }
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Render a frame with a pre-filtered set of chunk mesh references.
    /// This avoids copying meshes and allows the caller to do frustum culling.
    pub fn render_frame_refs(
        &self,
        camera: &Camera,
        sky: &DayNightCycle,
        chunk_meshes: &[&ChunkMesh],
    ) -> Result<(), wgpu::SurfaceError> {
        let uniform = camera.uniform();
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
        self.update_sky(sky);

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 1. Sky
            render_pass.set_pipeline(&self.sky_pipeline);
            render_pass.set_bind_group(0, &self.sky_bind_group, &[]);
            render_pass.draw(0..3, 0..1);

            // 2. Terrain chunks
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_atlas.bind_group, &[]);
            render_pass.set_bind_group(2, &self.terrain_sky_bind_group, &[]);

            for mesh in chunk_meshes {
                if mesh.index_count == 0 {
                    continue;
                }
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

/// Create a depth texture and return its view.
fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth_texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}
