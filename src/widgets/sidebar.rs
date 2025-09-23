use crate::prelude::*;
use super::sidebar_primary_list::SidebarPrimaryList;
use super::sidebar_category_list::SidebarCategoryList;

mod implementation {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/pt/timarques/european_choice/sidebar.ui")]
    pub struct Sidebar {
        #[template_child(id = "sidebar-primary-list")]
        pub primary_list: TemplateChild<SidebarPrimaryList>,
        #[template_child(id = "sidebar-category-list")]
        pub category_list: TemplateChild<SidebarCategoryList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "Sidebar";
        type Type = super::Sidebar;
        type ParentType = adw::NavigationPage;

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
    }

    impl WidgetImpl for Sidebar {}
    impl NavigationPageImpl for Sidebar {}
}

glib::wrapper! {
    pub struct Sidebar(ObjectSubclass<implementation::Sidebar>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Sidebar {

    pub fn primary_list(&self) -> &SidebarPrimaryList {
        &self.imp().primary_list
    }

    pub fn category_list(&self) -> &SidebarCategoryList {
        &self.imp().category_list
    }

    pub fn deactivate_rows(&self) {
        self.primary_list().deactivate_rows();
    }

    pub fn clear_changes(&self) {
        self.primary_list().search_row().clear_search();
        self.primary_list().country_row().select_default_item();
        self.category_list().select_first();
    }

}