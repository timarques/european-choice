use crate::prelude::*;
use crate::models::Country;
use std::cell::{RefCell, Cell};

mod implementation {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/pt/timarques/european_choice/sidebar_country_item.ui")]
    #[properties(wrapper_type = super::SidebarCountryItem)]
    pub struct SidebarCountryItem {
        #[template_child(id = "sidebar-country-item-image")]
        pub image: TemplateChild<gtk::Image>,

        #[property(get, set)]
        pub flag: RefCell<Option<String>>,
        #[property(get, set)]
        pub label: RefCell<String>,
        #[property(get, set)]
        pub caption: RefCell<String>,
        #[property(get, set)]
        pub caption_visible: Cell<bool>,
        #[property(get, set)]
        pub index: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarCountryItem {
        const NAME: &'static str = "SidebarCountryItem";
        type Type = super::SidebarCountryItem;
        type ParentType = gtk::Box;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
            Self::Type::ensure_type();
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SidebarCountryItem {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_image();
        }
    }
    
    impl WidgetImpl for SidebarCountryItem {}
    impl BoxImpl for SidebarCountryItem {}
}

glib::wrapper! {
    pub struct SidebarCountryItem(ObjectSubclass<implementation::SidebarCountryItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl SidebarCountryItem {
    fn setup_image(&self) {
        self.connect_flag_notify(|item| {
            let implementation = item.imp();
            if let Some(icon_name) = item.flag() {
                implementation.image.set_icon_name(Some(&icon_name));
                implementation.image.set_visible(true);
            } else {
                implementation.image.set_visible(false);
            }
        });
    }

    pub fn new(label: &str, caption: &str, icon: Option<&str>) -> Self {
        glib::Object::builder::<Self>()
            .property("flag", icon)
            .property("caption", caption)
            .property("label", label)
            .property("caption_visible", false)
            .build()
    }

    pub fn from_country(country: Country) -> Self {
        glib::Object::builder::<Self>()
            .property("flag", Some(country.slug()))
            .property("caption", "Country")
            .property("label", country.display_name())
            .property("caption_visible", false)
            .property("index", country as u32)
            .build()
    }
}