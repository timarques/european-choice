use super::super::prelude::*;
use super::overview_page::OverviewPage;
use super::sidebar::Sidebar;

use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/pt/timarques/european_choice/main_page.ui")]
    #[properties(wrapper_type = super::MainPage)]
    pub struct MainPage {
        #[template_child(id = "main-page-split-view")]
        pub split_view: TemplateChild<adw::NavigationSplitView>,
        #[template_child(id = "main-page-sidebar")]
        pub sidebar: TemplateChild<Sidebar>,
        #[template_child(id = "main-page-overview")]
        pub overview: TemplateChild<OverviewPage>,

        #[property(get, set)]
        pub title: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainPage {
        const NAME: &'static str = "MainPage";
        type Type = super::MainPage;
        type ParentType = adw::NavigationPage;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MainPage {

        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_overview();
            self.obj().setup_sidebar();
        }

    }
    impl WidgetImpl for MainPage {}
    impl NavigationPageImpl for MainPage {}
}

glib::wrapper! {
    pub struct MainPage(ObjectSubclass<imp::MainPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MainPage {

    fn setup_overview(&self) {
        let this_weak = self.downgrade();
        self.overview().connect_active_group_changed(move |_, group| {
            if let Some(this) = this_weak.upgrade() {
                this.sidebar().category_list().select_row_by_index(group.index() as usize);
            }
        });
    }

    fn setup_sidebar(&self) {
        let this_weak = self.downgrade();
        self.sidebar().category_list().connect_row_selected(move |_, row_index, _| {
            if let Some(this) = this_weak.upgrade() {
                this.overview().scroll_to_group_index(row_index);
            }
        });
    }

    pub fn sidebar(&self) -> &Sidebar {
        &self.imp().sidebar
    }

    pub fn overview(&self) -> &OverviewPage {
        &self.imp().overview
    }

    pub fn set_collapse(&self, collapse: bool) {
        self.imp().split_view.set_collapsed(collapse);
    }

}