use blue_engine::{
    wgpu, Camera, EnginePlugin, ObjectStorage, Renderer, Window as Win, DEPTH_FORMAT,
};

pub use egui;

/// The egui plugin
pub struct EGUI {
    pub context: egui::Context,
    pub platform: egui_winit::State,
    pub renderer: egui_wgpu::renderer::Renderer,
    pub full_output: Option<egui::FullOutput>,
}

impl EGUI {
    /// Creates the egui context and platform details
    pub fn new(event_loop: &blue_engine::EventLoop<()>, renderer: &mut Renderer) -> Self {
        let platform = egui_winit::State::new(event_loop);
        let format = renderer
            .surface
            .as_ref()
            .unwrap()
            .get_supported_formats(&renderer.adapter)[0];

        let renderer =
            egui_wgpu::renderer::Renderer::new(&renderer.device, format, Some(DEPTH_FORMAT), 1);

        Self {
            context: Default::default(),
            platform,
            renderer,
            full_output: None,
        }
    }

    pub fn ui<F: FnOnce(&egui::Context)>(&mut self, callback: F, window: &Win) {
        let raw_input = self.platform.take_egui_input(&window);

        self.full_output = Some(self.context.run(raw_input, callback));
    }
}

impl EnginePlugin for EGUI {
    /// updates the inputs and events
    fn update_events(
        &mut self,
        _renderer: &mut Renderer,
        _window: &Win,
        _objects: &mut ObjectStorage,
        _events: &blue_engine::Event<()>,
        _input: &blue_engine::InputHelper,
        _camera: &mut Camera,
    ) {
        match _events {
            blue_engine::Event::WindowEvent { event, .. } => {
                //? has a return, maybe useful in the future
                let _ = self.platform.on_event(&self.context, event);
            }
            _ => {}
        }
    }

    fn update(
        &mut self,
        renderer: &mut blue_engine::Renderer,
        window: &blue_engine::Window,
        _objects: &mut ObjectStorage,
        _camera: &mut blue_engine::Camera,
        _input: &blue_engine::InputHelper,
        encoder: &mut blue_engine::CommandEncoder,
        view: &blue_engine::TextureView,
    ) {
        if self.full_output.is_some() {
            let full_output = self.full_output.as_ref().unwrap();

            self.platform.handle_platform_output(
                &window,
                &self.context,
                full_output.platform_output.clone(),
            );

            let paint_jobs = self.context.tessellate(full_output.shapes.clone());

            let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [renderer.config.width, renderer.config.height],
                pixels_per_point: self.platform.pixels_per_point(),
            };

            for (id, image_delta) in &full_output.textures_delta.set {
                self.renderer
                    .update_texture(&renderer.device, &renderer.queue, *id, image_delta);
            }
            self.renderer.update_buffers(
                &renderer.device,
                &renderer.queue,
                encoder,
                &paint_jobs,
                &screen_descriptor,
            );

            {
                let mut render_pass =
                    encoder.begin_render_pass(&blue_engine::RenderPassDescriptor {
                        label: Some("Render pass"),
                        color_attachments: &[Some(blue_engine::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: blue_engine::Operations {
                                load: blue_engine::LoadOp::Load,
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &renderer.depth_buffer.1,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });

                self.renderer
                    .render(&mut render_pass, &paint_jobs, &screen_descriptor);
            }
        }
    }
}

// ===============================================================================================

struct TriangleRenderResources {
    pub shader: wgpu::RenderPipeline,
    pub vertex_buffer: blue_engine::VertexBuffers,
    pub texture: wgpu::BindGroup,
    pub uniform: blue_engine::UniformBuffers,
    pub default_data: (
        blue_engine::Textures,
        blue_engine::Shaders,
        blue_engine::UniformBuffers,
    ),
    pub camera_data: wgpu::BindGroup,
}

impl TriangleRenderResources {
    pub fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_bind_group(0, &self.default_data.0, &[]);
        render_pass.set_pipeline(&self.default_data.1);
        render_pass.set_bind_group(1, &self.camera_data, &[]);

        // Draw our triangle!
        let i = self;
        render_pass.set_pipeline(&i.shader);
        render_pass.set_bind_group(0, &i.texture, &[]);

        render_pass.set_bind_group(2, &i.uniform, &[]);

        render_pass.set_vertex_buffer(0, i.vertex_buffer.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            i.vertex_buffer.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..i.vertex_buffer.length, 0, 0..1);
    }
}

pub struct EmbeddedRender {}
impl EmbeddedRender {
    pub fn new(
        object: &mut blue_engine::Object,
        cc: &mut Renderer,
        renderer: &mut egui_wgpu::Renderer,
    ) -> Option<Self> {
        let buffers = object.update_and_return(cc).unwrap();

        let camera_data = cc
            .build_uniform_buffer(&vec![cc.build_uniform_buffer_part(
                "Camera Uniform",
                blue_engine::utils::default_resources::DEFAULT_MATRIX_4,
            )])
            .unwrap();

        let default_texture = cc
            .build_texture(
                "Default Texture",
                blue_engine::TextureData::Bytes(
                    blue_engine::utils::default_resources::DEFAULT_TEXTURE.to_vec(),
                ),
                blue_engine::header::TextureMode::Clamp,
                //crate::header::TextureFormat::PNG
            )
            .unwrap();

        let default_texture_2 = cc
            .build_texture(
                "Default Texture",
                blue_engine::TextureData::Bytes(
                    blue_engine::utils::default_resources::DEFAULT_TEXTURE.to_vec(),
                ),
                blue_engine::header::TextureMode::Clamp,
            )
            .unwrap();

        let default_uniform = cc
            .build_uniform_buffer(&vec![
                cc.build_uniform_buffer_part(
                    "Transformation Matrix",
                    blue_engine::utils::default_resources::DEFAULT_MATRIX_4,
                ),
                cc.build_uniform_buffer_part(
                    "Color",
                    blue_engine::uniform_type::Array4 {
                        data: blue_engine::utils::default_resources::DEFAULT_COLOR,
                    },
                ),
            ])
            .unwrap();

        let default_shader = cc
            .build_shader(
                "Default Shader",
                blue_engine::utils::default_resources::DEFAULT_SHADER.to_string(),
                Some(&default_uniform.1),
                blue_engine::ShaderSettings::default(),
            )
            .unwrap();

        renderer
            .paint_callback_resources
            .insert(TriangleRenderResources {
                shader: buffers.2,
                texture: default_texture,
                vertex_buffer: buffers.0,
                uniform: buffers.1,
                default_data: (default_texture_2, default_shader, default_uniform.0),
                camera_data: camera_data.0,
            });

        Some(Self {})
    }

    pub fn prepare(
        &mut self,
        object: &mut blue_engine::Object,
        brenderer: &mut blue_engine::Renderer,
        renderer: &mut egui_wgpu::Renderer,
        camera_data: blue_engine::UniformBuffers,
    ) {
        let object_pipeline = object.update_and_return(brenderer).unwrap();

        let resources: &mut TriangleRenderResources =
            renderer.paint_callback_resources.get_mut().unwrap();

        resources.vertex_buffer = object_pipeline.0;
        resources.uniform = object_pipeline.1;
        resources.shader = object_pipeline.2;
        resources.camera_data = camera_data;
    }

    pub fn paint(&mut self, ui: &mut egui::Ui) {
        let space = ui.available_size();

        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(space.x - 5f32, space.y - 5f32),
            egui::Sense::drag(),
        );

        let cb = egui_wgpu::CallbackFn::new().paint(
            move |_info, render_pass, paint_callback_resources| {
                let resources: &TriangleRenderResources = paint_callback_resources.get().unwrap();
                resources.paint(render_pass);
            },
        );

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(cb),
        };

        ui.painter().add(callback);
    }
}
