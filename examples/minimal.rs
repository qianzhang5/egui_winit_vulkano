// Copyright (c) 2021 Okko Hakola
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

#![allow(clippy::eq_op)]
use egui::{ScrollArea, TextEdit, TextStyle};
use egui_winit_vulkano::{Gui, GuiConfig};
use std::error::Error;
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    window::{VulkanoWindows, WindowDescriptor},
};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::WindowId,
};

fn sized_text(ui: &mut egui::Ui, text: impl Into<String>, size: f32) {
    ui.label(egui::RichText::new(text).size(size));
}

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(&event_loop);

    event_loop.run_app(&mut app)
}

struct App {
    context: VulkanoContext,
    windows: VulkanoWindows,
    window_id: Option<WindowId>,
    gui: Option<Gui>,
    code: String,
}

impl App {
    fn new(_event_loop: &EventLoop<()>) -> Self {
        let context = VulkanoContext::new(VulkanoConfig::default());
        let windows = VulkanoWindows::default();
        let code = CODE.to_owned();
        Self { context, windows, code, gui: None, window_id: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let App { context, windows, .. } = self;
        self.window_id =
            Some(windows.create_window(event_loop, context, &WindowDescriptor::default(), |ci| {
                ci.image_format = vulkano::format::Format::B8G8R8A8_UNORM;
                ci.min_image_count = ci.min_image_count.max(2);
            }));

        self.gui = Some({
            let renderer = windows.get_renderer_mut(self.window_id.unwrap()).unwrap();
            Gui::new(
                event_loop,
                renderer.surface(),
                renderer.graphics_queue(),
                renderer.swapchain_format(),
                GuiConfig::default(),
                None,
            )
        });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id_: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.window_id.unwrap() != window_id_ {
            return;
        }
        let App { windows, gui, code, .. } = self;
        let gui = gui.as_mut().expect("oops");
        let redraw = {
            let window = windows.get_window(window_id_).unwrap();
            let response = gui.update(window, &event);
            if response.consumed {
                return;
            }
            response.repaint
        };
        let renderer = windows.get_renderer_mut(window_id_).unwrap();
        match event {
            WindowEvent::RedrawRequested => {
                gui.immediate_ui(|gui| {
                    let ctx = gui.context();
                    egui::CentralPanel::default().show(&ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add(egui::widgets::Label::new("Hi there!"));
                            sized_text(ui, "Rich Text", 32.0);
                        });
                        ui.separator();
                        ui.columns(2, |columns| {
                            ScrollArea::vertical().id_salt("source").show(&mut columns[0], |ui| {
                                ui.add(TextEdit::multiline(code).font(TextStyle::Monospace));
                            });
                            ScrollArea::vertical().id_salt("rendered").show(
                                &mut columns[1],
                                |ui| {
                                    egui_demo_lib::easy_mark::easy_mark(ui, code);
                                },
                            );
                        });
                    });
                });
                match renderer.acquire(Some(std::time::Duration::from_secs(1)), |_| {}) {
                    Ok(future) => {
                        let after_future =
                            gui.draw_on_image(future, renderer.swapchain_image_view());
                        renderer.present(after_future, true);
                    }
                    Err(vulkano::VulkanError::OutOfDate) => {
                        renderer.resize();
                    }
                    Err(e) => panic!("Failed to acquire swapchain future: {}", e),
                };
            }
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::Resized(_size) => renderer.resize(),
            WindowEvent::ScaleFactorChanged { .. } => renderer.resize(),
            _ => {}
        }
        if redraw {
            renderer.window().request_redraw();
        }
    }
}

const CODE: &str = r"
# Some markup
```
let mut gui = Gui::new(&event_loop, renderer.surface(), None, renderer.queue(), SampleCount::Sample1);
```
";
