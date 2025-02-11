// Copyright (c) 2021 Okko Hakola
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.
use egui_winit_vulkano::{Gui, GuiConfig};
use std::error::Error;
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    window::{VulkanoWindows, WindowDescriptor},
};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::WindowId,
};

// Simply create egui demo apps to test everything works correctly.
// Creates two windows with different color formats for their swapchain.

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(&event_loop);

    event_loop.run_app(&mut app)
}

struct App {
    context: VulkanoContext,
    windows: VulkanoWindows,

    window_id1: Option<WindowId>,
    window_id2: Option<WindowId>,
    gui1: Option<Gui>,
    gui2: Option<Gui>,
    demo_app1: egui_demo_lib::DemoWindows,
    demo_app2: egui_demo_lib::DemoWindows,
    egui_test2: egui_demo_lib::ColorTest,
    egui_test1: egui_demo_lib::ColorTest,
}

impl App {
    fn new(_event_loop: &EventLoop<()>) -> Self {
        let context = VulkanoContext::new(VulkanoConfig::default());
        let windows = VulkanoWindows::default();
        let demo_app1 = egui_demo_lib::DemoWindows::default();
        let demo_app2 = egui_demo_lib::DemoWindows::default();
        let egui_test1 = egui_demo_lib::ColorTest::default();
        let egui_test2 = egui_demo_lib::ColorTest::default();
        Self {
            context,
            windows,
            gui1: None,
            gui2: None,
            window_id1: None,
            window_id2: None,
            demo_app1,
            demo_app2,
            egui_test1,
            egui_test2,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let App { context, windows, .. } = self;
        self.window_id1 =
            Some(windows.create_window(event_loop, context, &WindowDescriptor::default(), |ci| {
                ci.image_format = vulkano::format::Format::B8G8R8A8_UNORM;
                ci.min_image_count = ci.min_image_count.max(2);
            }));
        self.window_id2 =
            Some(windows.create_window(event_loop, context, &WindowDescriptor::default(), |ci| {
                ci.image_format = vulkano::format::Format::B8G8R8A8_UNORM;
                ci.min_image_count = ci.min_image_count.max(2);
            }));

        self.gui1 = Some({
            let renderer = windows.get_renderer_mut(self.window_id1.unwrap()).unwrap();
            Gui::new(
                event_loop,
                renderer.surface(),
                renderer.graphics_queue(),
                renderer.swapchain_format(),
                GuiConfig::default(),
                None,
            )
        });
        self.gui2 = Some({
            let renderer = windows.get_renderer_mut(self.window_id2.unwrap()).unwrap();
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
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let windows = &mut self.windows;
        let (gui, demo_app, egui_test) = if window_id == self.window_id1.unwrap() {
            (self.gui1.as_mut().unwrap(), &mut self.demo_app1, &mut self.egui_test1)
        } else if window_id == self.window_id2.unwrap() {
            (self.gui2.as_mut().unwrap(), &mut self.demo_app2, &mut self.egui_test2)
        } else {
            return;
        };

        let redraw = {
            let window = windows.get_window(window_id).unwrap();
            let response = gui.update(window, &event);
            if response.consumed {
                return;
            }
            response.repaint
        };
        let renderer = windows.get_renderer_mut(window_id).unwrap();
        match event {
            WindowEvent::RedrawRequested => {
                gui.immediate_ui(|gui| {
                    let ctx = gui.context();
                    demo_app.ui(&ctx);

                    egui::Window::new("Colors").vscroll(true).show(&ctx, |ui| {
                        egui_test.ui(ui);
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
