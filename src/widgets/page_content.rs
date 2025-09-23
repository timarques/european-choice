use super::super::prelude::*;

use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/pt/timarques/european_choice/page_content.ui")]
    #[properties(wrapper_type = super::PageContent)]
    pub struct PageContent {
        #[template_child(id = "page-content-scrolled-window")]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[property(get, set)]
        pub title: RefCell<String>,
        #[property(get, set)]
        pub subtitle: RefCell<String>,
        #[property(get, set)]
        pub content: RefCell<Option<gtk::Widget>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PageContent {
        const NAME: &'static str = "PageContent";
        type Type = super::PageContent;
        type ParentType = adw::Bin;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PageContent {}
    impl WidgetImpl for PageContent {}
    impl BinImpl for PageContent {}
}

glib::wrapper! {
    pub struct PageContent(ObjectSubclass<imp::PageContent>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PageContent {

    pub fn scrolled_window(&self) -> &gtk::ScrolledWindow {
        &self.imp().scrolled_window
    }

}