use crate::prelude::*;
use crate::models::{Category, Product};
use std::cell::RefCell;
use glib::{GString, Properties};

// PRODUCT_LIST

mod product_list {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/pt/timarques/eu_alternatives/product_list.ui")]
    pub struct ProductList {
        #[template_child(id = "product-list-preferences")]
        pub preferences: TemplateChild<adw::PreferencesPage>,
        pub groups: RefCell<Vec<ProductListGroup>>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProductList {
        const NAME: &'static str = "ProductList";
        type Type = super::ProductList;
        type ParentType = adw::NavigationPage;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    impl ObjectImpl for ProductList {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for ProductList {}
    impl NavigationPageImpl for ProductList {}
}

glib::wrapper! {
    pub struct ProductList(ObjectSubclass<product_list::ProductList>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ProductList {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn add_category(&self, category: &Category) {
        let group = ProductListGroup::from_category(category);
        self.add(group);
    }

    pub fn add(&self, group: ProductListGroup) {
        self.imp().groups.borrow_mut().push(group);
    }
}

impl Default for ProductList {
    fn default() -> Self {
        Self::new()
    }
}

// PRODUCT_LIST_GROUP

mod product_list_group {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/pt/timarques/eu_alternatives/product_list_group.ui")]
    #[properties(wrapper_type = super::ProductListGroup)]
    pub struct ProductListGroup {
        pub rows: RefCell<Vec<ProductListRow>>,
        
        #[property(get, set)]
        pub title: RefCell<GString>,
        #[property(get, set)]
        pub description: RefCell<GString>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProductListGroup {
        const NAME: &'static str = "ProductListGroup";
        type Type = super::ProductListGroup;
        type ParentType = adw::PreferencesGroup;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    impl ObjectImpl for ProductListGroup {
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
            obj.connect_title_notify(|group| {
                group.set_title(group.title());
            });
            obj.connect_description_notify(|group| {
                group.set_description(group.description());
            });
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for ProductListGroup {}
    impl PreferencesGroupImpl for ProductListGroup {}
}

glib::wrapper! {
    pub struct ProductListGroup(ObjectSubclass<product_list_group::ProductListGroup>)
        @extends adw::PreferencesGroup, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ProductListGroup {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn from_category(category: &Category) -> Self {
        glib::Object::builder::<Self>()
            .property("title", &category.name)
            .property("description", &category.description)
            .build()
    }

    pub fn add_product(&self, product: &Product) {
        let row = ProductListRow::from_product(product);
        self.add(row);
    }

    pub fn add(&self, row: ProductListRow) {
        PreferencesGroupExt::add(self, &row);
        self.imp().rows.borrow_mut().push(row);
    }
}

// PRODUCT_LIST_ROW

mod product_list_row {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/pt/timarques/eu_alternatives/product_list_row.ui")]
    #[properties(wrapper_type = super::ProductListRow)]
    pub struct ProductListRow {
        #[template_child(id = "product-logo")]
        pub logo_widget: TemplateChild<gtk::Image>,
        #[template_child(id = "product-country-flag")]
        pub country_flag_widget: TemplateChild<gtk::Image>,
        #[template_child(id = "product-country-name")]
        pub country_name_widget: TemplateChild<gtk::Label>,
        
        #[property(get, set)]
        pub name: RefCell<GString>,
        #[property(get, set)]
        pub description: RefCell<GString>,
        #[property(get, set)]
        pub logo: RefCell<GString>,
        #[property(get, set, name = "country-name")]
        pub country_name: RefCell<GString>,
        #[property(get, set, name = "country-flag")]
        pub country_flag: RefCell<GString>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProductListRow {
        const NAME: &'static str = "ProductListRow";
        type Type = super::ProductListRow;
        type ParentType = adw::ActionRow;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    impl ObjectImpl for ProductListRow {
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
            obj.connect_name_notify(|row| {
                row.set_title(&row.name());
            });
            obj.connect_description_notify(|row| {
                row.set_subtitle(&row.description());
            });
            obj.connect_logo_notify(|row| {
                row.imp().logo_widget.set_icon_name(Some(&row.logo()));
            });
            obj.connect_country_name_notify(|row| {
                row.imp().country_name_widget.set_label(&row.country_name());
            });
            obj.connect_country_flag_notify(|row| {
                row.imp().country_flag_widget.set_icon_name(Some(&row.country_flag()));
            });
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for ProductListRow {}
    impl ListBoxRowImpl for ProductListRow {}
    impl ActionRowImpl for ProductListRow {}
    impl PreferencesRowImpl for ProductListRow {}
}

glib::wrapper! {
    pub struct ProductListRow(ObjectSubclass<product_list_row::ProductListRow>)
        @extends adw::ActionRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl ProductListRow {

    pub fn from_product(product: &Product) -> Self {
        let mut builder = glib::Object::builder::<Self>()
            .property("name", &product.name)
            .property("description", &product.description);

        if let Some(logo) = product.logo {
            builder = builder.property("logo", logo);
        }

        if let Some(country) = product.country {
            builder = builder
                .property("country-name", country.name())
                .property("country-flag", country.icon());
        }

        builder.build()
    }

}