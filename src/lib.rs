use context::Context;

extern crate glium;

pub mod context;
pub mod timer;
pub mod io;

use egui_winit::winit::event_loop::{EventLoop, ControlFlow};
pub use timer::Timer;

use glium::{
    glutin::{
        window::WindowBuilder,
        ContextBuilder,
        event::Event,
    },
    *
};

/// Implement this trait to run it with `run` or `run_with_context`!
pub trait Application {
    /// This function is called after everything is setup but before the first frame is rendered
    fn init(&mut self, ctx: &mut Context);
    /// Called every frame to give the application a chance to update and render, the timer provides information like the time since the last frame and the current frame rate
    fn update(&mut self, t: &Timer, ctx: &mut Context);
    /// Called when the window is requested to close
    fn close(&mut self);
    /// Called a number of times between each frame with all new incoming events for the application
    fn handle_event(&mut self, ctx: &mut Context, event: &Event<()>);
}

/// Create and run a glium window for this application
/// 
/// # Arguments
/// 
/// * `mut app: Application` - the application you want to run with glium
/// * `wb: WindowBuilder` - Settings on how the window should be shaped/sized/positioned/resizable etc
pub fn run<A: 'static + Application>(app: A, wb: WindowBuilder) {
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
/// * `mut app: Application` - the application you want to run with glium
/// * `mut context: Context` - A glium_app Context containing a Display, Egui object and io managers
/// * `event_loop: EventLoop<()>` - The EventLoop for the window
pub fn run_with_context<A: 'static + Application>(mut app: A, mut context: Context, event_loop: EventLoop<()>) {
    let mut t = Timer::new();

    t.reset();
    event_loop.run(move |ev, _, control_flow| {

        // Handle our own events
        let mut events_cleared = false;
        use glutin::event::{self, Event::*};


        match &ev {
            MainEventsCleared => {
                events_cleared = true;
            }
            NewEvents(cause) => match cause {
                event::StartCause::Init => {
                    app.init(&mut context);
                }
                _ => {}
            },
            WindowEvent{ window_id: _, event: event::WindowEvent::CloseRequested } => {
                app.close();
                *control_flow = ControlFlow::Exit;
            },
            WindowEvent{ window_id: _, event } => {
                context.gui.on_event(event);
                context.handle_event(&ev);
                app.handle_event(&mut context, &ev);
            },
            _ => {
                context.handle_event(&ev);
                app.handle_event(&mut context, &ev);
            }
        }

        if !events_cleared {
            return;
        }

        // Update
        match t.go() {
            None => {}
            Some(_) => {
                app.update(&t, &mut context);

                context.mouse.next_frame();
                context.keyboard.next_frame();
            }
        }
    });
}