/*
 * Blue Engine by Elham Aryanpur
 *
 * Basic GUI example
 *
 * Licensed under Apache-2.0
*/

// EmbeddedRender enables rendering objects inside egui
use blue_engine_egui::{egui, EmbeddedRender};

// Basic imports
use blue_engine::{
    header::{Engine, WindowDescriptor},
    primitive_shapes::cube,
};

fn main() {
    // Initialize the engine with default settings
    let mut engine = Engine::new(WindowDescriptor::default()).expect("win");

    // the object to display
    cube("cube", &mut engine.renderer, &mut engine.objects).expect("Couldn't create the cube");

    // Start the egui context
    let mut gui_context = blue_engine_egui::EGUI::new(&engine.event_loop, &mut engine.renderer);

    // here we only need object temporarily
    let mut custom_rendering = {
        let mut object = engine.objects.get_mut("cube").unwrap();
        // this will not render the object in the background
        object.is_visible = false;
        // create our instance of graphics
        EmbeddedRender::new(&mut object, &mut engine.renderer, &mut gui_context.renderer).unwrap()
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

            // Get our object
            let cube = objects.get_mut("cube").unwrap();
            // and get current camera unifrom data
            let camera_data = camera.update_view_projection_and_return(renderer).unwrap();
            // and prepare the data for our graphics
            custom_rendering.prepare(cube, renderer, &mut egui_plugin.renderer, camera_data);
            // ui function will provide the context
            egui_plugin.ui(
                |ctx| {
                    // This window will contain our graphics
                    egui::Window::new("title").resizable(true).show(ctx, |ui| {
                        // We make a canvas to paint our graphics
                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            // Paint our graphics
                            custom_rendering.paint(ui);
                        });

                        // to allocate space that is available after resize
                        ui.allocate_space(ui.available_size());
                    });

                    // We can also do our other GUI stuff as always,
                    egui::Window::new("Pick Color")
                        .resizable(true)
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Pick a color");
                                ui.color_edit_button_rgba_unmultiplied(&mut color);
                            });
                        });
                },
                &window,
            );

            // we can normally apply changes to our graphics
            cube.set_uniform_color(color[0], color[1], color[2], color[3])
                .unwrap();

            // and even other settings
            let camx = start.elapsed().unwrap().as_secs_f32().sin() * radius;
            let camy = start.elapsed().unwrap().as_secs_f32().sin() * radius;
            let camz = start.elapsed().unwrap().as_secs_f32().cos() * radius;
            camera
                .set_position(camx, camy, camz)
                .expect("Couldn't update the camera eye");
        })
        .expect("Error during update loop");
}
