use crate::prelude::*;
use super::product_list::ProductList;
use super::sidebar::Sidebar;

use gtk::gio::{ActionGroup, ActionMap};

mod window {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/pt/timarques/eu_alternatives/window.ui")]
    pub struct Window {
        #[template_child(id = "navigation-view")]
        pub navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child(id = "overlay-split-view")]
        pub overlay_split_view: TemplateChild<adw::OverlaySplitView>,
        #[template_child(id = "product-list")]
        pub product_list: TemplateChild<ProductList>,
        #[template_child(id = "sidebar")]
        pub sidebar: TemplateChild<Sidebar>,
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
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<window::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager, ActionMap, ActionGroup;
}

impl Window {
    pub fn new(application: &adw::Application) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    pub fn sidebar(&self) -> &Sidebar {
        &self.imp().sidebar
    }

}