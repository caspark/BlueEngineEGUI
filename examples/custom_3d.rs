/*
 * Blue Engine by Elham Aryanpur
 *
 * Basic GUI example
 *
 * Licensed under Apache-2.0
*/

use std::{num::NonZeroU64, sync::Arc};

// Gui is a trait that you'll be using to add your UI
use blue_engine_egui::egui;

// Basic imports
use blue_engine::{
    header::{Engine, ObjectSettings, WindowDescriptor},
    primitive_shapes::cube,
    utils::default_resources::{DEFAULT_COLOR, DEFAULT_MATRIX_4, DEFAULT_SHADER, DEFAULT_TEXTURE},
    wgpu::{self, util::DeviceExt},
    Object, ObjectStorage, Renderer, UniformBuffers, Vertex, VertexBuffers,
};

struct TriangleRenderResources {
    pub shader: wgpu::RenderPipeline,
    pub vertex_buffer: VertexBuffers,
    pub texture: wgpu::BindGroup,
    pub uniform: UniformBuffers,
    pub default_data: (
        blue_engine::Textures,
        blue_engine::Shaders,
        blue_engine::UniformBuffers,
    ),
    pub camera_data: wgpu::BindGroup,
}

impl TriangleRenderResources {
    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
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

pub struct Custom3d {}
impl Custom3d {
    pub fn new(
        object: &mut Object,
        cc: &mut Renderer,
        renderer: &mut egui_wgpu::Renderer,
    ) -> Option<Self> {
        let buffers = object.update_and_return(cc).unwrap();

        let camera_data = cc
            .build_uniform_buffer(&vec![
                cc.build_uniform_buffer_part("Camera Uniform", DEFAULT_MATRIX_4)
            ])
            .unwrap();

        let default_texture = cc
            .build_texture(
                "Default Texture",
                blue_engine::TextureData::Bytes(DEFAULT_TEXTURE.to_vec()),
                blue_engine::header::TextureMode::Clamp,
                //crate::header::TextureFormat::PNG
            )
            .unwrap();

        let default_texture_2 = cc
            .build_texture(
                "Default Texture",
                blue_engine::TextureData::Bytes(DEFAULT_TEXTURE.to_vec()),
                blue_engine::header::TextureMode::Clamp,
            )
            .unwrap();

        let default_uniform = cc
            .build_uniform_buffer(&vec![
                cc.build_uniform_buffer_part("Transformation Matrix", DEFAULT_MATRIX_4),
                cc.build_uniform_buffer_part(
                    "Color",
                    blue_engine::uniform_type::Array4 {
                        data: DEFAULT_COLOR,
                    },
                ),
            ])
            .unwrap();

        let default_shader = cc
            .build_shader(
                "Default Shader",
                DEFAULT_SHADER.to_string(),
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

    fn prepare(
        &mut self,
        object: &mut Object,
        brenderer: &mut blue_engine::Renderer,
        renderer: &mut egui_wgpu::Renderer,
        camera_data: crate::UniformBuffers,
    ) {
        let object_pipeline = object.update_and_return(brenderer).unwrap();

        let resources: &mut TriangleRenderResources =
            renderer.paint_callback_resources.get_mut().unwrap();

        resources.vertex_buffer = object_pipeline.0;
        resources.uniform = object_pipeline.1;
        resources.shader = object_pipeline.2;
        resources.camera_data = camera_data;
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, _response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        let cb = egui_wgpu::CallbackFn::new().paint(
            move |_info, render_pass, paint_callback_resources| {
                let resources: &TriangleRenderResources = paint_callback_resources.get().unwrap();
                resources.paint(render_pass);
            },
        );

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }
}

fn main() {
    // Initialize the engine with default settings
    let mut engine = Engine::new(WindowDescriptor::default()).expect("win");

    cube("trig", &mut engine.renderer, &mut engine.objects);

    // Start the egui context
    let mut gui_context = blue_engine_egui::EGUI::new(&engine.event_loop, &mut engine.renderer);

    let mut custom_rendering = {
        let mut object = engine.objects.get_mut("trig").unwrap();
        object.is_visible = false;
        Custom3d::new(&mut object, &mut engine.renderer, &mut gui_context.renderer).unwrap()
    };

    // We add the gui as plugin, which runs once before everything else to fetch events, and once during render times for rendering and other stuff
    engine.plugins.push(Box::new(gui_context));

    let mut color = [1f32, 1f32, 1f32, 1f32];
    let radius = 5f32;
    let start = std::time::SystemTime::now();

    // Update loop
    engine
        .update_loop(move |renderer, window, objects, _, camera, plugins| {
            // obtain the plugin
            let egui_plugin = plugins[0]
                // downcast it to obtain the plugin
                .downcast_mut::<blue_engine_egui::EGUI>()
                .expect("Plugin not found");

            let trig = objects.get_mut("trig").unwrap();
            let camera_data = camera.update_view_projection_and_return(renderer).unwrap();
            custom_rendering.prepare(trig, renderer, &mut egui_plugin.renderer, camera_data);
            // ui function will provide the context
            egui_plugin.ui(
                |ctx| {
                    egui::Window::new("title").show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Pick a color");
                            ui.color_edit_button_rgba_unmultiplied(&mut color);
                        });

                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            custom_rendering.custom_painting(ui);
                        });
                    });
                },
                &window,
            );

            trig.set_uniform_color(color[0], color[1], color[2], color[3])
                .unwrap();

            let camx = start.elapsed().unwrap().as_secs_f32().sin() * radius;
            let camy = start.elapsed().unwrap().as_secs_f32().sin() * radius;
            let camz = start.elapsed().unwrap().as_secs_f32().cos() * radius;
            camera
                .set_position(camx, camy, camz)
                .expect("Couldn't update the camera eye");
        })
        .expect("Error during update loop");
}
