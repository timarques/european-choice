use super::super::prelude::*;
use super::super::application::Application;
use super::navigation::Navigation;

use gtk::gio::{ActionGroup, ActionMap};
use std::sync::OnceLock;
use std::cell::Cell;

const WINDOW_SIZE_CHANGED_SIGNAL: &str = "state-changed";

#[derive(Default, Clone, Copy, Debug, glib::Boxed)]
#[boxed_type(name = "WindowSize")]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
    pub maximized: bool
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/pt/timarques/european_choice/window.ui")]
    pub struct Window {
        #[template_child(id = "window-toast-overlay")]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child(id = "window-navigation")]
        pub navigation: TemplateChild<Navigation>,
        #[template_child(id = "window-click-gesture")]
        pub click_gesture: TemplateChild<gtk::GestureClick>,
        #[template_child(id = "window-breakpoint")]
        pub breakpoint: TemplateChild<adw::Breakpoint>,

        pub size: Cell<WindowSize>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            if cfg!(windows) {
                self.obj().set_decorated(false);
            }
            
            self.obj().setup_click_gesture();
            self.obj().setup_breakpoint();
            self.obj().setup_size_monitoring();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<[glib::subclass::Signal; 1]> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                [
                    glib::subclass::Signal::builder(WINDOW_SIZE_CHANGED_SIGNAL)
                        .param_types([WindowSize::static_type()])
                        .build(),
                ]
            })
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager, ActionMap, ActionGroup;
}

impl Window {

    fn set_pages_titles(&self, title: &str) {
        self.imp().navigation.main_page().set_title(title);
        self.imp().navigation.product_page().set_title(title);
    }

    fn setup_click_gesture(&self) {
        let this_weak = self.downgrade();
        self.imp().click_gesture.connect_pressed(move |_, _, _, _| {
            if let Some(this) = this_weak.upgrade() {
                this.navigation()
                    .main_page()
                    .sidebar()
                    .deactivate_rows();
            }
        });
    }

    fn setup_breakpoint(&self) {
        let this_weak = self.downgrade();
        self.imp().breakpoint.connect_apply(move |_| {
            if let Some(this) = this_weak.upgrade() {
                this.navigation().main_page().set_collapse(true);
            }
        });

        let this_weak = self.downgrade();
        self.imp().breakpoint.connect_unapply(move |_| {
            if let Some(this) = this_weak.upgrade() {
                this.navigation().main_page().set_collapse(false);
            }
        });
    }

    fn update_window_size(&self) {
        let (width, height) = self.default_size();

        if width > 0 && height > 0 {
            let mut size = self.imp().size.get();
            size.width = width.unsigned_abs();
            size.height = height.unsigned_abs();
            size.maximized = self.is_maximized();

            let previous_size = self.imp().size.get();

            if size.width != previous_size.width
                || size.height != previous_size.height
                || size.maximized != previous_size.maximized
            {
                self.imp().size.set(size);
                self.emit_by_name::<()>(WINDOW_SIZE_CHANGED_SIGNAL, &[&size]);
            }
        }
    }

    fn setup_size_monitoring(&self) {
        let initial_size = WindowSize {
            width: self.default_width().clamp(0, i32::MAX).cast_unsigned(),
            height: self.default_height().clamp(0, i32::MAX).cast_unsigned(),
            maximized: self.is_maximized(),
        };

        self.imp().size.set(initial_size);

        let this_weak = self.downgrade();
        self.connect_default_width_notify(move |_| {
            if let Some(this) = this_weak.upgrade() {
                this.update_window_size();
            }
        });

        let this_weak = self.downgrade();
        self.connect_default_height_notify(move |_| {
            if let Some(this) = this_weak.upgrade() {
                this.update_window_size();
            }
        });

        let this_weak = self.downgrade();
        self.connect_maximized_notify(move |_| {
            if let Some(this) = this_weak.upgrade() {
                this.update_window_size();
            }
        });
    }

    pub fn new(application: &Application, title: &str) -> Self {
        let this = glib::Object::builder::<Self>()
            .property("application", application)
            .build();

        this.set_pages_titles(title);
        this.present();
        this
    }

    pub fn navigation(&self) -> &Navigation {
        &self.imp().navigation
    }

    pub fn notify(&self, message: &str) {
        let toast = adw::Toast::new(message);
        self.imp().toast_overlay.add_toast(toast);
    }

    pub fn set_size(&self, size: WindowSize) {
        self.imp().size.set(size);
        self.set_default_size(size.width.cast_signed(), size.height.cast_signed());
        self.set_maximized(size.maximized);
    }

    pub fn connect_size_changed<F>(&self, callback: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, WindowSize) + 'static,
    {
        self.connect_local(WINDOW_SIZE_CHANGED_SIGNAL, false, move |values| {
            if let (Some(window), Some(size)) = (values[0].get::<Self>().ok(), values[1].get::<WindowSize>().ok()) {
                callback(&window, size);
            }
            None
        })
    }
}