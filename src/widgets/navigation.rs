use super::super::prelude::*;
use super::loading_page::LoadingPage;
use super::main_page::MainPage;
use super::product_page::ProductPage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationPage {
    Loading,
    Main,
    Product,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/pt/timarques/european_choice/navigation.ui")]
    pub struct Navigation {
        #[template_child(id = "navigation-view")]
        pub view: TemplateChild<adw::NavigationView>,
        #[template_child(id = "navigation-loading-page")]
        pub loading_page: TemplateChild<LoadingPage>,
        #[template_child(id = "navigation-main-page")]
        pub main_page: TemplateChild<MainPage>,
        #[template_child(id = "product-page")]
        pub product_page: TemplateChild<ProductPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Navigation {
        const NAME: &'static str = "Navigation";
        type Type = super::Navigation;
        type ParentType = adw::Bin;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    impl ObjectImpl for Navigation {}
    impl WidgetImpl for Navigation {}
    impl BinImpl for Navigation {}
}

glib::wrapper! {
    pub struct Navigation(ObjectSubclass<imp::Navigation>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Navigation {
    pub fn loading_page(&self) -> &LoadingPage {
        &self.imp().loading_page
    }

    pub fn main_page(&self) -> &MainPage {
        &self.imp().main_page
    }

    pub fn product_page(&self) -> &ProductPage {
        &self.imp().product_page
    }

    pub fn push_page(&self, page: NavigationPage) {
        let view: &adw::NavigationView = &self.imp().view;

        let widget: &adw::NavigationPage = match page {
            NavigationPage::Loading => self.loading_page().upcast_ref(),
            NavigationPage::Main => self.main_page().upcast_ref(),
            NavigationPage::Product => self.product_page().upcast_ref(),
        };

        view.push(widget);
    }

    pub fn replace_with_page(&self, page: NavigationPage) {
        let view: &adw::NavigationView = &self.imp().view;

        let widget: adw::NavigationPage = match page {
            NavigationPage::Loading => self.loading_page().clone().upcast(),
            NavigationPage::Main => self.main_page().clone().upcast(),
            NavigationPage::Product => self.product_page().clone().upcast(),
        };

        view.replace(&[widget]);
    }

    pub fn page(&self) -> Option<NavigationPage> {
        let view = &self.imp().view;
        let tag = view.visible_page()?.tag();

        if self.loading_page().tag() == tag {
            Some(NavigationPage::Loading)
        } else if self.main_page().tag() == tag {
            Some(NavigationPage::Main)
        } else if self.product_page().tag() == tag {
            Some(NavigationPage::Product)
        } else {
            None
        }
    }

    pub fn pop(&self) -> bool {
        self.imp().view.pop()
    }
}