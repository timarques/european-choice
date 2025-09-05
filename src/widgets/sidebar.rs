use glib::{GString, Properties};
use std::cell::RefCell;
use crate::prelude::*;

pub enum SidebarNode {
    Categories,
    Countries,
    About
}

mod sidebar {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/pt/timarques/eu_alternatives/sidebar.ui")]
    pub struct Sidebar {
        #[template_child(id = "sidebar-list")]
        pub list: TemplateChild<gtk::ListBox>,
        #[template_child(id = "sidebar-row-categories")]
        pub row_categories: TemplateChild<super::SidebarRow>,
        #[template_child(id = "sidebar-row-countries")]
        pub row_countries: TemplateChild<super::SidebarRow>,
        #[template_child(id = "sidebar-row-about")]
        pub row_about: TemplateChild<super::SidebarRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "Sidebar";
        type Type = super::Sidebar;
        type ParentType = gtk::Box;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    impl ObjectImpl for Sidebar {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for Sidebar {}
    impl BoxImpl for Sidebar {}
}

glib::wrapper! {
    pub struct Sidebar(ObjectSubclass<sidebar::Sidebar>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Sidebar {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn add_leaf(&self, node: SidebarNode) {
        
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

// SIDEBAR_ROW

mod sidebar_row {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/pt/timarques/eu_alternatives/sidebar_row.ui")]
    #[properties(wrapper_type = super::SidebarRow)]
    pub struct SidebarRow {
        #[template_child(id = "sidebar-row-icon")]
        pub icon_widget: TemplateChild<gtk::Image>,
        #[template_child(id = "sidebar-row-label")]
        pub label_widget: TemplateChild<gtk::Label>,
        #[template_child(id = "sidebar-row-revealer")]
        pub revealer: TemplateChild<gtk::Revealer>,

        #[property(get, set)]
        pub icon: RefCell<GString>,
        #[property(get, set)]
        pub label: RefCell<GString>,
        #[property(get, set, default = true)]
        pub expand: RefCell<bool>,
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

    impl ObjectImpl for SidebarRow {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            
            let obj = self.obj();
            obj.connect_icon_notify(|row| {
                row.imp().icon_widget.set_icon_name(Some(&row.icon()));
            });
            obj.connect_label_notify(|row| {
                row.imp().label_widget.set_label(&row.label());
            });
            obj.connect_expand_notify(|row| {
                row.imp().revealer.set_reveal_child(row.expand());
            });
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }
    
    impl ListBoxRowImpl for SidebarRow {}
    impl WidgetImpl for SidebarRow {}
}

glib::wrapper! {
    pub struct SidebarRow(ObjectSubclass<sidebar_row::SidebarRow>)
        @extends gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl SidebarRow {
    pub fn new(icon: &str, label: &str) -> Self {
        glib::Object::builder::<Self>()
            .property("icon", icon)
            .property("label", label)
            .build()
    }
}