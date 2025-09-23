use super::super::prelude::*;

use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/pt/timarques/european_choice/loading_page.ui")]
    #[properties(wrapper_type = super::LoadingPage)]
    pub struct LoadingPage {
        #[property(get, set)]
        pub spinning: Cell<bool>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LoadingPage {
        const NAME: &'static str = "LoadingPage";
        type Type = super::LoadingPage;
        type ParentType = adw::NavigationPage;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for LoadingPage {}
    impl WidgetImpl for LoadingPage {}
    impl NavigationPageImpl for LoadingPage {}
}

glib::wrapper! {
    pub struct LoadingPage(ObjectSubclass<imp::LoadingPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}