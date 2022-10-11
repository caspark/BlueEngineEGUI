use blue_engine::{Camera, EnginePlugin, Object, Renderer, Window as Win};

pub use egui;

/// Allows you to write UI code understandable by this library.
/// The only function is `update` function, passing all normal components as well as `ui`.
pub trait Gui {
    fn update(
        &mut self,
        _window: &Win,
        _renderer: &mut Renderer,
        _objects: &mut std::collections::HashMap<&'static str, Object>,
        _camera: &mut Camera,
        _input: &blue_engine::InputHelper,
        _plugin_data_storage: &mut std::collections::HashMap<&'static str, Box<dyn std::any::Any>>,
        ui: &egui::Context,
    );
}

/// The egui plugin
pub struct EGUI {
    pub context: egui::Context,
    pub platform: egui_winit::State,
    pub renderer: egui_wgpu::renderer::RenderPass,
    pub gui: Box<dyn Gui>,
}

impl EGUI {
    /// Creates the egui context and platform details
    pub fn new(
        event_loop: &blue_engine::EventLoop<()>,
        renderer: &mut Renderer,
        gui: Box<dyn Gui>,
    ) -> Self {
        let platform = egui_winit::State::new(event_loop);
        let renderer = egui_wgpu::renderer::RenderPass::new(
            &renderer.device,
            renderer
                .surface
                .as_ref()
                .unwrap()
                .get_supported_formats(&renderer.adapter)[0],
            1,
        );

        Self {
            context: Default::default(),
            platform,
            renderer,
            gui,
        }
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
                self.platform.on_event(&self.context, event);
            }
            _ => {}
        }
    }

    fn update(
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
        //if renderer.surface.is_some() {
        let raw_input = self.platform.take_egui_input(&window);
        //}

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

        //self.render_pass
        //    .update_texture(&renderer.device, &renderer.queue, &tdelta);
        for (id, image_delta) in &textures_delta.set {
            self.renderer
                .update_texture(&renderer.device, &renderer.queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            &renderer.device,
            &renderer.queue,
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
                depth_stencil_attachment: None,
            });

            self.renderer.execute_with_renderpass(
                &mut render_pass,
                &paint_jobs,
                &screen_descriptor,
            );
        }
    }
}

// ===============================================================================================
