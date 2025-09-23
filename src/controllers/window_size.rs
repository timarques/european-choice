use super::super::prelude::*;
use super::super::widgets::WindowSize as WindowSizeData;
use super::super::ui::Ui;

use std::rc::{Rc, Weak};

const WIDTH_KEY: &str = "window-width";
const HEIGHT_KEY: &str = "window-height";
const MAXIMIZED_KEY: &str = "window-maximized";

struct State {
    ui: Ui,
    settings: gtk::gio::Settings
}

pub struct WeakWindowSize {
    state: Weak<State>
}

impl WeakWindowSize {
    pub fn upgrade(&self) -> Option<WindowSize> {
        self.state.upgrade().map(|state| WindowSize { state })
    }
}

pub struct WindowSize {
    state: Rc<State>
}

impl WindowSize {

    pub fn new(ui: Ui, settings: gtk::gio::Settings) -> Self {
        let state = Rc::new(State { ui, settings });
        let controller = Self { state };
        controller.setup_window_size_changed();
        controller.apply_saved_size();
        controller
    }

    fn setup_window_size_changed(&self) {
        let controller_weak = self.downgrade();
        self.state.ui.window().connect_size_changed(move |window, size| {
            if let Some(controller) = controller_weak.upgrade()
                && let Err(error) = controller.save_window_size(size)
            {
                window.notify(&error.to_string());
            }
        });
    }

    fn load_saved_size(&self) -> WindowSizeData {
        WindowSizeData {
            width: self.state.settings.int(WIDTH_KEY).clamp(0, i32::MAX).unsigned_abs(),
            height: self.state.settings.int(HEIGHT_KEY).clamp(0, i32::MAX).unsigned_abs(),
            maximized: self.state.settings.boolean(MAXIMIZED_KEY),
        }
    }

    fn apply_saved_size(&self) {
        let saved_size = self.load_saved_size();
        self.state.ui.window().set_size(saved_size);
    }

    fn save_window_size(&self, window_size: WindowSizeData) -> Result<()> {
        self.state.settings.set_int(WIDTH_KEY, window_size.width.cast_signed())?;
        self.state.settings.set_int(HEIGHT_KEY, window_size.height.cast_signed())?;
        self.state.settings.set_boolean(MAXIMIZED_KEY, window_size.maximized)?;
        Ok(())
    }

    pub fn downgrade(&self) -> WeakWindowSize {
        let state = Rc::downgrade(&self.state);
        WeakWindowSize { state }
    }

}