use super::super::prelude::*;
use super::page_content::PageContent;
use super::product_row::ProductRow;

use std::cell::{Cell, RefCell};
use std::sync::OnceLock;
use std::collections::HashMap;
use glib::subclass::Signal;

const ROW_ACTIVATED_SIGNAL: &str = "row-activated";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, glib::Enum)]
#[enum_type(name = "ProductRowType")]
pub enum ProductRowType {
    Country,
    Website,
    Category,
}

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/pt/timarques/european_choice/product_page.ui")]
    #[properties(wrapper_type = super::ProductPage)]
    pub struct ProductPage {
        #[template_child(id = "product-page-content")]
        pub content: TemplateChild<PageContent>,
        #[template_child(id = "product-page-websites-group")]
        pub websites_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child(id = "product-page-country-list-box")]
        pub country_list_box: TemplateChild<gtk::ListBox>,
        #[template_child(id = "product-page-categories-group")]
        pub categories_group: TemplateChild<adw::PreferencesGroup>,

        #[property(get, set)]
        pub index: Cell<u32>,
        #[property(get, set)]
        pub logo: RefCell<Option<String>>,
        #[property(get, set)]
        pub name: RefCell<String>,
        #[property(get, set)]
        pub description: RefCell<String>,

        pub rows_by_type: RefCell<HashMap<ProductRowType, Vec<ProductRow>>>,
    }

    impl Default for ProductPage {
        fn default() -> Self {
            Self {
                rows_by_type: RefCell::new(HashMap::from_iter([
                    (ProductRowType::Country, Vec::new()),
                    (ProductRowType::Website, Vec::new()),
                    (ProductRowType::Category, Vec::new())
                ])),
                content: TemplateChild::default(),
                websites_group: TemplateChild::default(),
                categories_group: TemplateChild::default(),
                country_list_box: TemplateChild::default(),
                index: Cell::new(0),
                logo: RefCell::new(None),
                name: RefCell::new(String::new()),
                description: RefCell::new(String::new()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProductPage {
        const NAME: &'static str = "ProductPage";
        type Type = super::ProductPage;
        type ParentType = adw::NavigationPage;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
            Self::Type::ensure_type();
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ProductPage {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<[Signal; 1]> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                [
                    glib::subclass::Signal::builder(ROW_ACTIVATED_SIGNAL)
                        .param_types([ProductRow::static_type(), ProductRowType::static_type()])
                        .build()
                ]
            })
        }
    }

    impl WidgetImpl for ProductPage {}
    impl NavigationPageImpl for ProductPage {}
}

glib::wrapper! {
    pub struct ProductPage(ObjectSubclass<imp::ProductPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ProductPage {
    fn add_row_to_container(&self, row: &ProductRow, row_type: ProductRowType) {
        let imp = self.imp();
        match row_type {
            ProductRowType::Country => imp.country_list_box.append(row),
            ProductRowType::Website => imp.websites_group.add(row),
            ProductRowType::Category => imp.categories_group.add(row),
        }
    }

    fn remove_row_from_container(&self, row: &ProductRow, row_type: ProductRowType) {
        let imp = self.imp();
        match row_type {
            ProductRowType::Country => imp.country_list_box.remove(row),
            ProductRowType::Website => imp.websites_group.remove(row),
            ProductRowType::Category => imp.categories_group.remove(row),
        }
    }

    fn setup_row_activation(&self, row: &ProductRow, row_type: ProductRowType) -> glib::SignalHandlerId {
        let this_weak = self.downgrade();
        row.connect_activated(move |activated_row| {
            if let Some(this) = this_weak.upgrade() {
                this.emit_by_name::<()>(ROW_ACTIVATED_SIGNAL, &[&activated_row, &row_type]);
            }
        })
    }

    pub fn remove_all_rows(&self) {
        let imp = self.imp();
        let mut rows = imp.rows_by_type.borrow_mut();

        for (row_type, rows) in rows.iter_mut() {
            for row in rows.drain(..) {
                self.remove_row_from_container(&row, *row_type);
            }
        }
    }

    pub fn append_row(&self, row: ProductRow, row_type: ProductRowType) {
        let imp = self.imp();
        let mut rows_by_type = imp.rows_by_type.borrow_mut();
        let rows  = rows_by_type.get_mut(&row_type).unwrap();

        self.add_row_to_container(&row, row_type);
        self.setup_row_activation(&row, row_type);
        rows.push(row);
    }

    pub fn connect_row_activated<F>(&self, callback: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, &ProductRow, ProductRowType) + 'static
    {
        self.connect_local(ROW_ACTIVATED_SIGNAL, true, move |values| {
            let this = values[0].get::<Self>().unwrap();
            let row = values[1].get::<&ProductRow>().unwrap();
            let row_type = values[2].get::<ProductRowType>().unwrap();
            callback(&this, row, row_type);
            None
        })
    }
}