use super::prelude::*;
use super::constants;
use super::widgets::WindowWidget;
use super::ui::Ui;
use super::repository::Repository;
use super::populator::Populator;
use super::search_engine::SearchEngine;
use super::controllers::{
    SearchController,
    ProductActivationController,
    ProductRowActivationController,
    WindowSizeController,
    ActionsController
};

use std::cell::OnceCell;

mod implementation {
    use super::*;

    pub struct Application {
        #[cfg(schemas_installed)]
        pub settings: gtk::gio::Settings,

        pub repository: Repository,
        pub ui: OnceCell<Ui>,
        pub search_controller: OnceCell<SearchController>,
        pub product_activation_controller: OnceCell<ProductActivationController>,
        pub product_row_activation_controller: OnceCell<ProductRowActivationController>,
        pub window_size_controller: OnceCell<WindowSizeController>,
        pub actions_controller: OnceCell<ActionsController>,
    }

    impl Default for Application {
        fn default() -> Self {
            Self {
                #[cfg(schemas_installed)]
                settings: gtk::gio::Settings::new(constants::APP_ID),

                repository: Repository::new(&constants::APP_CATALOG),
                ui: OnceCell::new(),
                search_controller: OnceCell::new(),
                product_activation_controller: OnceCell::new(),
                product_row_activation_controller: OnceCell::new(),
                window_size_controller: OnceCell::new(),
                actions_controller: OnceCell::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "MyApplication";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self) {
            self.parent_activate();
            self.obj().setup_activation();
        }

        fn startup(&self) {
            self.parent_startup();
            super::Application::setup_startup();
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<implementation::Application>)
        @extends adw::Application, gtk::Application, gtk::gio::Application,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap;
}

impl Default for Application {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", constants::APP_ID)
            .property("resource-base-path", constants::APP_PREFIX)
            .property("flags", gtk::gio::ApplicationFlags::default())
            .build()
    }
}

impl Application {
    pub fn new() -> Self {
        Self::default()
    }

    fn setup_startup() {
        glib::set_application_name(constants::APP_TITLE);
        glib::set_prgname(Some(constants::APP_NAME));
        gtk::gio::resources_register_include_impl(constants::APP_RESOURCES).unwrap();
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::PreferDark);

        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_resource(&format!("{prefix}/style.css", prefix = constants::APP_PREFIX));

        let display = gtk::gdk::Display::default().unwrap();
        gtk::style_context_add_provider_for_display(&display, &css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        if let Some(directory) = constants::GSETTINGS_SCHEMA_DIR {
            unsafe { std::env::set_var("GSETTINGS_SCHEMA_DIR", directory) };
        }
    }

    fn setup_activation(&self) {
        let window = WindowWidget::new(self, constants::APP_TITLE);
        let ui = Ui::new(window);
        Populator::populate(&ui, self.imp().repository);

        self.setup_controllers(&ui);

        ui.activate();
        self.imp().ui.set(ui).ok().unwrap();
    }

    fn setup_controllers(&self, ui: &Ui) {
        let repository = self.imp().repository;

        let search_controller = SearchController::new(ui.clone(), SearchEngine::new(repository));
        let product_activation_controller = ProductActivationController::new(ui.clone(), repository);
        let product_row_activation_controller = ProductRowActivationController::new(ui.clone(), repository);

        let actions_controller = ActionsController::new(self.clone(), search_controller.clone());

        self.imp().search_controller.set(search_controller).ok().unwrap();
        self.imp().product_activation_controller.set(product_activation_controller).ok().unwrap();
        self.imp().product_row_activation_controller.set(product_row_activation_controller).ok().unwrap();
        self.imp().actions_controller.set(actions_controller).ok().unwrap();

        #[cfg(schemas_installed)]
        {
            let window_size_controller = WindowSizeController::new(ui.clone(), self.imp().settings.clone());
            self.imp().window_size_controller.set(window_size_controller).ok().unwrap();
        }
    }

    pub fn run(&self) -> Result<()> {
        let result = ApplicationExtManual::run(self);
        if matches!(result, adw::glib::ExitCode::FAILURE) {
            bail!("Application exited with code {}", result.get());
        }

        Ok(())
    }

}