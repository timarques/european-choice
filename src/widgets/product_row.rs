use super::super::prelude::*;
use super::super::models::{Category, Country};

use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/pt/timarques/european_choice/product_row.ui")]
    #[properties(wrapper_type = super::ProductRow)]
    pub struct ProductRow {
        #[template_child(id = "product-row-image")]
        pub image: TemplateChild<gtk::Image>,

        #[property(get, set)]
        pub icon: RefCell<Option<String>>,
        #[property(get, set)]
        pub icon_visible: Cell<bool>,
        #[property(get, set)]
        pub icon_white: Cell<bool>,
        #[property(get, set)]
        pub index: Cell<u32>,
        #[property(get, set)]
        pub feature_subtitle: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProductRow {
        const NAME: &'static str = "ProductRow";
        type Type = super::ProductRow;
        type ParentType = adw::ActionRow;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ProductRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_classes();
        }
    }

    impl WidgetImpl for ProductRow {}
    impl ListBoxRowImpl for ProductRow {}
    impl PreferencesRowImpl for ProductRow {}
    impl ActionRowImpl for ProductRow {}
}

glib::wrapper! {
    pub struct ProductRow(ObjectSubclass<imp::ProductRow>)
        @extends adw::ActionRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ProductRow {
    fn setup_classes(&self) {
        self.connect_notify_local(Some("icon-white"), |this, _| {
            this.update_icon_white_class();
        });

        self.connect_notify_local(Some("feature-subtitle"), |this, _| {
            this.update_feature_subtitle_class();
        });
    }

    fn update_icon_white_class(&self) {
        let image = self.imp().image.get();
        if self.icon_white() {
            image.add_css_class("icon-white");
        } else {
            image.remove_css_class("icon-white");
        }
    }

    fn update_feature_subtitle_class(&self) {
        if self.feature_subtitle() {
            self.add_css_class("feature-subtitle");
        } else {
            self.remove_css_class("feature-subtitle");
        }
    }

    pub fn new(
        title: &str,
        subtitle: Option<&str>,
        icon: Option<&str>,
        index: usize
    ) -> Self {
        let escaped_title = glib::markup_escape_text(title);

        let mut builder = glib::Object::builder::<Self>()
            .property("title", escaped_title);

        if let Some(subtitle) = subtitle {
            let escaped_subtitle = glib::markup_escape_text(subtitle);
            builder = builder.property("subtitle", escaped_subtitle);
        }

        builder
            .property("icon", icon)
            .property("icon_visible", icon.is_some())
            .property("index", index as u32)
            .build()
    }

    pub fn from_category(category: &Category, index: usize) -> Self {
        let this = Self::new(category.name, Some(category.summary), Some(category.icon), index);
        this.set_icon_white(true);
        this
    }

    pub fn from_website(caption: &str, url: &str, index: usize) -> Self {
        let this = Self::new(caption, Some(url), None, index);
        this.set_feature_subtitle(true);
        this
    }

    pub fn from_country(country: Country) -> Self {
        let this = Self::new("Country", Some(country.display_name()), Some(country.slug()), country as usize);
        this.set_feature_subtitle(true);
        this
    }

}