use context::Context;

extern crate glium;

pub mod context;
pub mod timer;
pub mod io;

use egui_winit::winit::event_loop::EventLoop;
pub use timer::Timer;

pub use glium::{
    glutin::{
        dpi::{PhysicalSize, Size},
        window::WindowBuilder,
        ContextBuilder,
    },
    *
};

/// Implement this trait then box up your object to run it with `run` or `run_with_context`!
pub trait Application {
    /// This function is called after everything is setup but before the first frame is rendered
    fn init(&mut self, ctx: &mut Context);
    /// Called every frame to give the application a chance to update, the timer provides information like the time since the last frame and the current frame rate
    fn update(&mut self, t: &Timer, ctx: &mut Context);
    /// Called every frame after `update`
    fn render(&mut self, ctx: &mut Context);
    /// Called when the window is requested to close
    fn close(&mut self);
}

/// Create and run a glium window for this application
/// 
/// # Arguments
/// 
/// * `mut app: Box<dyn Application>` - the application you want to run with glium
/// * `wb: WindowBuilder` - Settings on how the window should be shaped/sized/positioned/resizable etc
pub fn run(mut app: Box<dyn Application>, wb: WindowBuilder) {
    let (ctx, el) = create(wb);
    run_with_context(app, ctx, el);
}

/// Create a `glium_app::Context` and `EventLoop<()>` which are required to run a glium_app Application
/// 
/// # Returns
/// 
/// * `(Context, EventLoop<()>)` - The Context and EventLoop needed to run the application with `run_with_context`
/// The EventLoop is pretty useless, but this function is generally used to get access to the Context and more specifically the Display inside it, if perhaps your Application needs to create a renderer which needs access to a Display
pub fn create(wb: WindowBuilder) -> (Context, EventLoop<()>) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let cb = ContextBuilder::new().with_vsync(false);
    let display = Display::new(wb, cb, &event_loop).expect("Failed to open Display!");

    let egui_glium = egui_glium::EguiGlium::new(&display);

    let context: Context = Context::new(display, egui_glium);

    (context, event_loop)
}

/// Run a glium_app Application with a provided Context and EventLoop (usually obtained from `create`)
/// 
/// # Arguments
/// 
/// * `mut app: Box<dyn Application>` - the application you want to run with glium
/// * `mut context: Context` - A glium_app Context containing a Display, Egui object and io managers
/// * `event_loop: EventLoop<()>` - The EventLoop for the window
pub fn run_with_context(mut app: Box<dyn Application>, mut context: Context, event_loop: EventLoop<()>) {
    let mut t = Timer::new();

    t.reset();
    event_loop.run(move |ev, _, control_flow| {

        use glutin::event::WindowEvent;

        // Handle our own events
        let mut events_cleared = false;
        use glutin::event::{Event::*, *};
        match &ev {
            glutin::event::Event::WindowEvent { event, .. } => 
            {
                let consume = context.gui.on_event(&event);

                match event {
                    WindowEvent::CloseRequested => {
                        app.close();
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                        ..
                    } => {
                        context.mouse.update_pos((position.x as i32, position.y as i32));
                    }
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                        ..
                    } => {
                        let mut mbutton: u16 = 0;
                        match button {
                            MouseButton::Left => {
                                mbutton = 0;
                            }
                            MouseButton::Middle => {
                                mbutton = 1;
                            }
                            MouseButton::Right => {
                                mbutton = 2;
                            }
                            MouseButton::Other(bnum) => {
                                if bnum > &(9 as u16) {
                                    return;
                                }
                                mbutton = *bnum;
                            }
                        }
                        let mut pressed = false;
                        if state == &ElementState::Pressed {
                            pressed = true;
                        }
                        if pressed {
                            context.mouse.press_button(mbutton as usize);
                        } else {
                            context.mouse.release_button(mbutton as usize);
                        }
                    }
                    WindowEvent::MouseWheel {
                        device_id, delta, ..
                    } => match delta {
                        MouseScrollDelta::LineDelta(y, x) => {
                            context.mouse.scroll((*x, *y));
                        }
                        _ => {}
                    },
                    WindowEvent::KeyboardInput {
                        device_id,
                        input,
                        is_synthetic,
                        ..
                    } => match input {
                        KeyboardInput {
                            scancode: _,
                            state,
                            virtual_keycode,
                            ..
                        } => match virtual_keycode {
                            None => {}
                            Some(key) => {
                                if state == &ElementState::Pressed {
                                    context.keyboard.press(*key);
                                } else {
                                    context.keyboard.release(*key);
                                }
                            }
                        },
                    },
                    _ => {}
            }
            },
            MainEventsCleared => {
                events_cleared = true;
            }
            RedrawEventsCleared => {}
            NewEvents(cause) => match cause {
                StartCause::Init => {
                    app.init(&mut context);
                }
                _ => {}
            },
            _ => {}
        }

        if !events_cleared {
            return;
        }

        // Update
        match t.go() {
            None => {}
            Some(_) => {
                app.update(&t, &mut context);
                app.render(&mut context);

                context.mouse.next_frame();
                context.keyboard.next_frame();
            }
        }
    });
}