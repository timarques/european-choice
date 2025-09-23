use crate::prelude::*;
use crate::models::Category;
use std::cell::{RefCell, Cell};
use glib::GString;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/pt/timarques/european_choice/sidebar_row.ui")]
    #[properties(wrapper_type = super::SidebarRow)]
    pub struct SidebarRow {
        #[property(get, set)]
        pub icon: RefCell<GString>,
        #[property(get, set)]
        pub label: RefCell<GString>,
        #[property(get, set, default = true)]
        pub expand: Cell<bool>,
        #[property(get, construct_only)]
        pub index: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarRow {
        const NAME: &'static str = "SidebarRow";
        type Type = super::SidebarRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SidebarRow {}
    impl ListBoxRowImpl for SidebarRow {}
    impl WidgetImpl for SidebarRow {}
}

glib::wrapper! {
    pub struct SidebarRow(ObjectSubclass<imp::SidebarRow>)
        @extends gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl SidebarRow {
    pub fn new(icon: &str, label: &str, index: usize) -> Self {
        glib::Object::builder::<Self>()
            .property("icon", icon)
            .property("label", label)
            .property("index", index as u32)
            .build()
    }

    pub fn from_category(category: &Category, index: usize) -> Self {
        glib::Object::builder::<Self>()
            .property("icon", category.icon)
            .property("label", category.name)
            .property("index", index as u32)
            .build()
    }

}