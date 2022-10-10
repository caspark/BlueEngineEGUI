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
        ui: &egui::Context,
    );
}

/// The egui plugin
pub struct EGUI {
    pub platform: egui_winit_platform::Platform,
    pub render_pass: egui_wgpu_backend::RenderPass,
    pub start_time: std::time::Instant,
    pub gui: Box<dyn Gui>,
}

impl EGUI {
    /// Creates the egui context and platform details
    pub fn new(window: &Win, renderer: &mut Renderer, gui: Box<dyn Gui>) -> Self {
        let window_size = window.inner_size();

        let platform =
            egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
                physical_width: window_size.width as u32,
                physical_height: window_size.height as u32,
                scale_factor: window.scale_factor(),
                font_definitions: egui::FontDefinitions::default(),
                style: Default::default(),
            });

        let egui_rpass = egui_wgpu_backend::RenderPass::new(
            &renderer.device,
            renderer
                .surface
                .as_ref()
                .unwrap()
                .get_supported_formats(&renderer.adapter)[0],
            1,
        );

        let start_time = std::time::Instant::now();

        Self {
            platform,
            render_pass: egui_rpass,
            start_time,
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
        if _renderer.surface.is_some() {
            self.platform.handle_event(&_events);
        }
    }

    /// Updates the egui with custom renderpass and renders UI code
    fn update(
        &mut self,
        renderer: &mut Renderer,
        window: &Win,
        objects: &mut std::collections::HashMap<&'static str, Object>,
        camera: &mut Camera,
        encoder: &mut blue_engine::CommandEncoder,
        view: &blue_engine::TextureView,
    ) {
        self.platform
            .update_time(self.start_time.elapsed().as_secs_f64());

        self.platform.begin_frame();
        self.gui
            .update(&window, renderer, objects, camera, &self.platform.context());

        let full_output = self.platform.end_frame(Some(&window));
        let paint_jobs = self.platform.context().tessellate(full_output.shapes);

        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: renderer.config.width,
            physical_height: renderer.config.height,
            scale_factor: window.scale_factor() as f32,
        };
        let tdelta: egui::TexturesDelta = full_output.textures_delta;
        self.render_pass
            .add_textures(&renderer.device, &renderer.queue, &tdelta)
            .expect("add texture ok");
        self.render_pass.update_buffers(
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

            self.render_pass
                .execute_with_renderpass(&mut render_pass, &paint_jobs, &screen_descriptor)
                .unwrap();
        }
    }
}

// ===============================================================================================
