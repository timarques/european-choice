use std::rc::Rc;

use super::prelude::*;
use super::constants;
use super::widgets::{Window, ProductList, Sidebar, SidebarRow};
use super::repository::Repository;

struct ApplicationState {
    application: adw::Application,
    repository: Repository<'static>
}

pub struct Application {
    state: Rc<ApplicationState>,
}

impl Application {

    pub fn new() -> Self {
        let application = adw::Application::new(
            Some(constants::APP_ID),
            adw::gio::ApplicationFlags::default()
        );

        let repository = Repository::new(&constants::APP_CATALOG);
        let state = Rc::new(ApplicationState {
            application,
            repository
        });

        Self::setup_signals(&state);

        Self { state }
    }

    fn setup_signals(state: &Rc<ApplicationState>) {
        Self::setup_activate_event(state);
        Self::setup_startup_event(state);
    }

    fn setup_activate_event(state: &Rc<ApplicationState>) {
        let state_weak = Rc::downgrade(state);
        state.application.connect_activate(move |_application| {
            let Some(state) = state_weak.upgrade() else { return };
            let this = Self { state };
            this.setup_ui().unwrap();
        });
    }

    fn setup_startup_event(state: &Rc<ApplicationState>) {
        state.application.connect_startup(move |_application| {
            Self::setup_resources().unwrap();
        });
    }

    fn setup_ui(&self) -> Result<()> {
        let window = Window::new(&self.state.application);
        window.present();
        Ok(())
    }

    fn setup_resources() -> Result<()> {
        gtk::glib::set_application_name(constants::APP_TITLE);
        gtk::glib::set_prgname(Some(constants::APP_NAME));
        gtk::gio::resources_register_include_impl(constants::APP_RESOURCES)?;

        let icon_theme = gtk::IconTheme::default();
        icon_theme.add_resource_path(&format!("{}/icons", constants::APP_PREFIX));
        icon_theme.add_resource_path(&format!("{}/icons/scalable/actions", constants::APP_PREFIX));

        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_resource(&format!("{}/style.css", constants::APP_PREFIX));

        let style_manager = adw::StyleManager::default();
        style_manager.set_color_scheme(adw::ColorScheme::PreferDark);

        let display = gtk::gdk::Display::default().context("Failed to add style provider")?;

        gtk::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        Ok(())
    }

    fn populate_sidebar_categories(&self, sidebar: &Sidebar) -> Result<()> {
        for category in self.state.repository.categories() {
            
        }

        Ok(())
    }

    pub fn activate(&self) -> Result<()> {
        let result = self.state.application.run();
        if matches!(result, adw::glib::ExitCode::FAILURE) {
            bail!("Application exited with code {}", result.get());
        }

        Ok(())
    }

}