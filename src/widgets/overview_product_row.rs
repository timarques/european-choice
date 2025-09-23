use crate::prelude::*;
use crate::models::Product;
use std::cell::{RefCell, Cell};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/pt/timarques/european_choice/overview_product_row.ui")]
    #[properties(wrapper_type = super::OverviewProductRow)]
    pub struct OverviewProductRow {
        #[template_child(id = "overview-product-row-logo")]
        pub logo_image: TemplateChild<gtk::Image>,
        #[template_child(id = "overview-product-row-country")]
        pub country_image: TemplateChild<gtk::Image>,
        #[template_child(id = "overview-product-row-suffix")]
        pub suffix_box: TemplateChild<gtk::Box>,

        #[property(get, set)]
        pub name: RefCell<String>,
        #[property(get, set)]
        pub summary: RefCell<String>,
        #[property(get, set)]
        pub logo: RefCell<String>,
        #[property(get, set)]
        pub country: RefCell<Option<String>>,
        #[property(get, construct_only)]
        pub index: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OverviewProductRow {
        const NAME: &'static str = "OverviewProductRow";
        type Type = super::OverviewProductRow;
        type ParentType = adw::ActionRow;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for OverviewProductRow {}
    impl WidgetImpl for OverviewProductRow {}
    impl ListBoxRowImpl for OverviewProductRow {}
    impl ActionRowImpl for OverviewProductRow {}
    impl PreferencesRowImpl for OverviewProductRow {}
}

glib::wrapper! {
    pub struct OverviewProductRow(ObjectSubclass<imp::OverviewProductRow>)
        @extends adw::ActionRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl OverviewProductRow {

    pub fn new(name: &str, summary: &str, logo: &str, index: usize) -> Self {
        glib::Object::builder()
            .property("name", name)
            .property("summary", summary)
            .property("logo", logo)
            .property("index", index as u32)
            .build()
    }

    pub fn from_product(product: &Product, index: usize) -> Self {
        let escaped_name = glib::markup_escape_text(product.name);
        let escaped_summary = glib::markup_escape_text(product.summary);

        let this = Self::new(&escaped_name, &escaped_summary, product.logo, index);
        if let Some(country) = product.country {
            this.set_property("country", country.slug());
        }

        this.imp().suffix_box.set_visible(product.country.is_some());
        this
    }

}