use egui_glium::EguiGlium;
use egui_winit::winit::error::ExternalError;
use glium::Display;
use glium::glutin::event::Event;

use crate::io::{keyboard::Keyboard, mouse::Mouse};

pub struct Context {
    pub dis: Display,
    pub gui: EguiGlium,

    pub mouse: Mouse,
    pub keyboard: Keyboard,
}


impl Context {
    pub fn new(dis: Display, gui: EguiGlium) -> Context {

        Context {
            dis,
            gui,

            mouse: Mouse::new(),
            keyboard: Keyboard::new(),
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        match event {
            _ => {
                self.keyboard.handle_event(event);
                self.mouse.handle_event(event);
            }
        }
    }


    pub fn set_mouse_grabbed(&self, grabbed: bool) -> Result<(), ExternalError> {
        let gl_win = self.dis.gl_window();
        let win = gl_win.window();

        win.set_cursor_grab(grabbed)
    }

    pub fn set_mouse_visible(&self, visible: bool) {
        let gl_win = self.dis.gl_window();
        let win = gl_win.window();

        win.set_cursor_visible(visible);
    }

}