use std::sync::atomic::{AtomicU64, Ordering};

use egui::Context;


pub struct PersistentWindowManager<S> {
    windows: Vec<PersistentWindow<S>>,
}

static WINDOW_IDS: AtomicU64 = AtomicU64::new(0);

type PersistentWindowFunction<S> = Box<dyn FnMut(&u64, &mut Vec<PersistentWindow<S>>, &Context, &mut S) -> bool>;

pub struct PersistentWindow<S> {
    pub id: u64,
    pub function: PersistentWindowFunction<S>,
}

impl<S> PersistentWindow<S> {
    pub fn new(function: PersistentWindowFunction<S>) -> PersistentWindow<S> {
        PersistentWindow { id: WINDOW_IDS.fetch_add(1, Ordering::Relaxed), function }
    }
}

impl<S> PersistentWindowManager<S> {
    pub fn new() -> PersistentWindowManager<S> {
        PersistentWindowManager { windows: Vec::new() }
    }

    pub fn push(&mut self, window: PersistentWindow<S>) {
        self.windows.push(window);
    }

    pub fn render(&mut self, state: &mut S, gui_ctx: &Context) {
        let mut new_windows: Vec<PersistentWindow<S>> = Vec::new();

        self.windows.retain_mut(|window| {
            (window.function)(&window.id, &mut new_windows, gui_ctx, state)
        });

        self.windows.append(&mut new_windows);
    }
}