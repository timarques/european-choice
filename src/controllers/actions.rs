use super::super::prelude::*;
use super::super::controllers::SearchController;
use super::super::application::Application;

pub struct Actions {
    application: Application,
    search_controller: SearchController,
}

impl Actions {
    pub fn new(application: Application, search_controller: SearchController) -> Self {
        let this = Self { application, search_controller };
        this.setup_quit_action();
        this.setup_search_action();
        this
    }

    fn setup_quit_action(&self) {
        let quit_action = gtk::gio::SimpleAction::new("quit", None);
        self.connect_quit_handler(&quit_action);
        self.application.add_action(&quit_action);
        self.application.set_accels_for_action("app.quit", &["<Ctrl>q"]);
    }

    fn connect_quit_handler(&self, quit_action: &gtk::gio::SimpleAction) {
        let application_weak = self.application.downgrade();
        quit_action.connect_activate(move |_action, _| {
            if let Some(application) = application_weak.upgrade() {
                application.quit();
            }
        });
    }

    fn setup_search_action(&self) {
        let search_action = gtk::gio::SimpleAction::new("search", None);
        self.connect_search_handler(&search_action);
        self.application.add_action(&search_action);
        self.application.set_accels_for_action("app.search", &["<Ctrl>space"]);
    }

    fn connect_search_handler(&self, search_action: &gtk::gio::SimpleAction) {
        let search_controller_weak = self.search_controller.downgrade();
        search_action.connect_activate(move |_action, _| {
            if let Some(search_controller) = search_controller_weak.upgrade() {
                search_controller.activate();
            }
        });
    }

}