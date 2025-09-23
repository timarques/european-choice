use crate::prelude::*;
use super::sidebar_row::SidebarRow;

use std::cell::{Ref, RefCell};
use std::collections::HashMap;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/pt/timarques/european_choice/sidebar_category_list.ui")]
    pub struct SidebarCategoryList {
        #[template_child(id = "sidebar-category-list-box")]
        pub list_box: TemplateChild<gtk::ListBox>,

        pub rows: RefCell<HashMap<usize, SidebarRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarCategoryList {
        const NAME: &'static str = "SidebarCategoryList";
        type Type = super::SidebarCategoryList;
        type ParentType = adw::Bin;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    impl ObjectImpl for SidebarCategoryList {}
    impl WidgetImpl for SidebarCategoryList {}
    impl BinImpl for SidebarCategoryList {}
}

glib::wrapper! {
    pub struct SidebarCategoryList(ObjectSubclass<imp::SidebarCategoryList>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl SidebarCategoryList {

    pub fn append_row(&self, row: SidebarRow) -> usize {
        let index = row.index() as usize;
        let implementation = self.imp();
        implementation.list_box.append(&row);

        if implementation.rows.borrow().is_empty() {
            implementation.list_box.select_row(Some(&row));
        }

        let mut rows = implementation.rows.borrow_mut();
        rows.insert(index, row);

        index
    }

    pub fn select_row_by_index(&self, index: usize) -> bool {
        self.imp().rows.borrow().get(&index).is_some_and(|row| {
            self.imp().list_box.select_row(Some(row));
            true
        })
    }

    pub fn select_first(&self) -> bool {
        self.imp().list_box.row_at_index(0).is_some_and(|first_row| {
            self.imp().list_box.select_row(Some(&first_row));
            true
        })
    }

    pub fn rows(&self) -> Ref<HashMap<usize, SidebarRow>> {
        self.imp().rows.borrow()
    }

    pub fn show_all_rows(&self) {
        let rows = self.rows();

        for (_, row) in rows.iter() {
            row.set_visible(true);
        }
    }

    pub fn apply_row_filter<F>(&self, predicate: F) 
    where
        F: Fn(&SidebarRow) -> bool,
    {
        let rows = self.rows();

        for (_, row) in rows.iter() {
            let should_show_row = predicate(row);
            row.set_visible(should_show_row);
        }
    }

    pub fn connect_row_selected<F>(&self, callback: F)
    where
        F: Fn(&Self, usize, &SidebarRow) + 'static
    {
        let this_weak = self.downgrade();
        self.imp().list_box.connect_row_selected(move |_list, row| {
            if
                let Some(this) = this_weak.upgrade()
                && let Some(row) = row
                && let Some(row) = row.downcast_ref::<SidebarRow>()
            {
                let index = row.index() as usize;
                callback(&this, index, row);
            }
        });
    }
}