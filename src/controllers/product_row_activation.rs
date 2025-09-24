use super::super::prelude::*;
use super::super::ui::Ui;
use super::super::repository::Repository;
use super::super::widgets::{ProductRowType, ProductRowWidget, NavigationPage};
use super::super::models::{Product, Country};

use std::rc::{Rc, Weak};
use std::time::Duration;

const TIEMOUT_DURATION: Duration = Duration::from_millis(200);

struct State {
    ui: Ui,
    repository: Repository
}

struct WeakProductRowActivation {
    state: Weak<State>
}

impl WeakProductRowActivation {
    fn upgrade(&self) -> Option<ProductRowActivation> {
        self.state.upgrade().map(|state| ProductRowActivation { state })
    }
}

pub struct ProductRowActivation {
    state: Rc<State>
}

impl ProductRowActivation {

    pub fn new(ui: Ui, repository: Repository) -> Self {
        let state = State { ui, repository };
        let this = Self { state: Rc::new(state) };
        this.setup_rows_activation();
        this
    }

    fn setup_rows_activation(&self) {
        let this_weak = self.downgrade();
        self.state.ui.product_page().connect_row_activated(move |product_page, row, row_type| {
            if 
                let Some(this) = this_weak.upgrade()
                && let Some(product) = this.state.repository.product_by_index(product_page.index() as usize)
            {
                this.state.ui.sidebar().clear_changes();
                match row_type {
                    ProductRowType::Website => this.handle_website_activation(product, row),
                    ProductRowType::Category => this.handle_category_activation_with_debounce(row),
                    ProductRowType::Country => this.handle_country_activation_with_debounce(row),
                }
            }
        });
    }

    fn handle_err(&self, error: &anyhow::Error) {
        self.state.ui.window().notify(&error.to_string());
        eprintln!("Error: {error}");
    }

    fn handle_website_activation(&self, product: &Product, row: &ProductRowWidget) {
        let website_index = row.index() as usize;
        let website_url = product.websites[website_index].1;
        self.launch_uri(website_url);
    }

    fn launch_uri(&self, uri: &str) {
        let window = self.state.ui.window();
        let this_weak = self.downgrade();
        let uri_owned = uri.to_string();
        gtk::UriLauncher::new(&uri_owned).launch(Some(window), None::<&gtk::gio::Cancellable>, move |result| {
            if 
                let Err(e) = result
                && let Some(this) = this_weak.upgrade()
            {
                let error = anyhow!("Failed to open website: {uri_owned}").context(e);
                this.handle_err(&error);
            }
        });
    }

    fn handle_category_activation_with_debounce(&self, row: &ProductRowWidget) {
        let category_index = row.index() as usize;
        self.state.ui.navigation().replace_with_page(NavigationPage::Main);
        self.debounce_action(move |this| {
            this.handle_category_activation(category_index);
        });
    }

    fn handle_country_activation_with_debounce(&self, row: &ProductRowWidget) {
        let country_index = row.index() as usize;
        self.state.ui.navigation().replace_with_page(NavigationPage::Main);
        self.debounce_action(move |this| {
            this.handle_country_activation(country_index);
        });
    }

    fn debounce_action<F>(&self, action: F)
    where
        F: FnOnce(&Self) + 'static,
    {
        let this_weak = self.downgrade();
        glib::timeout_add_local_once(TIEMOUT_DURATION, move || {
            if let Some(this) = this_weak.upgrade() {
                action(&this);
            }
        });
    }

    fn handle_category_activation(&self, category_index: usize) {
        let overview_page = self.state.ui.overview_page();
        let active_group_index = overview_page.active_group_index().unwrap_or(0);
        if
            active_group_index != category_index
            && !overview_page.scroll_to_group_index(category_index)
        {
            let category = self.state.repository.category_by_index(category_index).unwrap();
            let error = anyhow!("Failed to scroll to group index {category_name}", category_name = category.name);
            self.handle_err(&error);
        }
    }

    fn handle_country_activation(&self, country_index: usize) {
        if !self.state.ui.country_row().select_item_by_index(country_index) {
            let country = Country::all()[country_index];
            let error = anyhow!("Failed to scroll to group index {country_display_name}", country_display_name = country.display_name());
            self.handle_err(&error);
        }
    }

    fn downgrade(&self) -> WeakProductRowActivation {
        let state = Rc::downgrade(&self.state);
        WeakProductRowActivation { state }
    }

}