use super::super::prelude::*;
use super::super::models::Category;
use super::overview_product_row::OverviewProductRow;

use std::cell::{Ref, RefCell, Cell};
use std::collections::HashMap;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/pt/timarques/european_choice/overview_product_group.ui")]
    #[properties(wrapper_type = super::OverviewProductGroup)]
    pub struct OverviewProductGroup {
        #[template_child(id = "overview-product-group-list-box")]
        pub list_box: TemplateChild<gtk::ListBox>,

        #[property(get, set)]
        pub title: RefCell<String>,
        #[property(get, set)]
        pub description: RefCell<String>,
        #[property(get, set)]
        pub index: Cell<u32>,

        pub rows: RefCell<HashMap<usize, OverviewProductRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OverviewProductGroup {
        const NAME: &'static str = "OverviewProductGroup";
        type Type = super::OverviewProductGroup;
        type ParentType = gtk::Box;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for OverviewProductGroup {}
    impl WidgetImpl for OverviewProductGroup {}
    impl BoxImpl for OverviewProductGroup {}
}

glib::wrapper! {
    pub struct OverviewProductGroup(ObjectSubclass<imp::OverviewProductGroup>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl OverviewProductGroup {

    pub fn new(title: &str, description: &str, index: usize) -> Self {
        let escaped_title = glib::markup_escape_text(title);
        let escaped_description = glib::markup_escape_text(description);

        glib::Object::builder()
            .property("title", escaped_title.as_str())
            .property("description", escaped_description.as_str())
            .property("index", index as u32)
            .build()
    }

    pub fn from_category(category: &Category, index: usize) -> Self {
        Self::new(category.name, category.description, index)
    }

    pub fn append_row(&self, row: OverviewProductRow) -> usize {
        let key = row.index() as usize;
        let implementation = self.imp();
        implementation.list_box.append(&row);

        let mut rows = implementation.rows.borrow_mut();
        rows.insert(key, row);

        key
    }

    pub fn rows(&self) -> Ref<'_, HashMap<usize, OverviewProductRow>> {
        self.imp().rows.borrow()
    }

    pub fn show_all_rows(&self) {
        let rows = self.rows();

        for (_, row) in rows.iter() {
            row.set_visible(true);
        }

        self.set_visible(true);
    }

    pub fn apply_row_filter<F>(&self, predicate: F)
    where
        F: Fn(&OverviewProductRow) -> bool,
    {
        let mut group_should_be_visible = false;
        let rows = self.rows();

        for (_, row) in rows.iter() {
            let should_show_row = predicate(row);
            row.set_visible(should_show_row);
            group_should_be_visible = group_should_be_visible || should_show_row;
        }

        self.set_visible(group_should_be_visible);
    }

}