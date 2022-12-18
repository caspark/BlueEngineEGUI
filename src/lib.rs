use blue_engine::{wgpu, Camera, EnginePlugin, Object, Renderer, Window as Win, DEPTH_FORMAT};

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
    pub fn new(
        event_loop: &blue_engine::EventLoop<()>,
        renderer: &mut Renderer,
    ) -> Self {
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
        _objects: &mut std::collections::HashMap<&'static str, Object>,
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
        _objects: &mut std::collections::HashMap<&'static str, blue_engine::Object>,
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

    /*fn update(
        &mut self,
        renderer: &mut blue_engine::Renderer,
        window: &blue_engine::Window,
        objects: &mut std::collections::HashMap<&'static str, blue_engine::Object>,
        camera: &mut blue_engine::Camera,
        input: &blue_engine::InputHelper,
        plugin_data_storage: &mut std::collections::HashMap<&'static str, Box<dyn std::any::Any>>,
        encoder: &mut blue_engine::CommandEncoder,
        view: &blue_engine::TextureView,
    ) {
        let raw_input = self.platform.take_egui_input(&window);

        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            ..
        } = self.context.run(raw_input, |context| {
            self.gui.update(
                &window,
                renderer,
                objects,
                camera,
                &input,
                plugin_data_storage,
                &context,
            );
        });

        self.platform
            .handle_platform_output(&window, &self.context, platform_output);

        let paint_jobs = self.context.tessellate(shapes);

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [renderer.config.width, renderer.config.height],
            pixels_per_point: self.platform.pixels_per_point(),
        };

        for (id, image_delta) in &textures_delta.set {
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
            let mut render_pass = encoder.begin_render_pass(&blue_engine::RenderPassDescriptor {
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

            self.renderer.render(
                &mut render_pass,
                &paint_jobs,
                &screen_descriptor,
            );
        }
    } */
}

// ===============================================================================================
