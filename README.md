# Blue Engine egui plugin

This is a plugin that adds [egui](egui.rs/) support to the [Blue Engine](https://githb.com/AryanpurTech/BlueEngine).

## Getting started

To get started, create a new struct that will contain your GUI code. For our example:

```rust
struct Counter {
    count: u16
}
```

Then We'll implement the `Gui` trait of the plugin:

```rust
impl Gui for MyGUI {
    // This is the starting point for your UI code, it passes almost all variables of the engine as well
    fn update(
        // for accessing the values
        &mut self,
        // We can add underscore to ones we don't use, so they won't emit warnings
        _window: &mut blue_engine::Window,
        _renderer: &mut blue_engine::Renderer,
        _objects: &mut std::collections::HashMap<&'static str, blue_engine::Object>,
        _camera: &mut blue_engine::Camera,
        ui: &blue_engine_egui::egui::Ui,
    ) {
        /* Your UI code goes here */
    }
}
```

And finally your EGUI code:

```rust
// Create a new egui window to contain the UI
blue_engine_egui::egui::Window::new("Counter Window").show(ui, |ui| {
    // Add a text to display the counter
    ui.text(format!("The count is at: {}", self.count));
    
    // + 1 per click
    if ui.button("Add 1").clicked() {
        self.count += 1;
    }
});
```

One more steps is left to be done to have the plugin working. We need to initialize the plugin before update loop:

```rust
let gui_context = blue_engine_egui::EGUI::new(&engine.window, &mut engine.renderer, Box::new(MyGui {count: 0}));
```

This will essentially initializes the egui and create things required to run the plugin. The engine will then run it twice, once before everything else to fetch all inputs and events, and then during render, so that it displays the GUI. And all that's left, is to add the plugin to the engine:

```rust
engine.plugins.push(Box::new(gui_context));
```

Congrats now you have a working GUI!

## Style Block

*The guide will come soon, it's cool I promise!*

## Examples

Check the [examples](https://github.com/AryanpurTech/BlueEngineEGUI/tree/master/examples) folder for potential UIs and as template for your new projects.

## Dependency justification

* `blue_engine`: Used obiously for exporting some components and struct declrations required to design the API
* `egui_wgpu_backend`: Used to assist in applying egui to wgpu graphics backend. Which is same graphics backend used in Blue Engine.
* `egui_winit_platform`: Support for Winit windowing. Which is same windowing system used in Blue Engine.
* `egui` and `epi`: The egui itself, required to obtain components and declrations for api design.
