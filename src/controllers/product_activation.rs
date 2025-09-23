use super::super::prelude::*;
use super::super::ui::Ui;
use super::super::models::Product;
use super::super::repository::Repository;
use super::super::widgets::{
    OverviewProductRowWidget,
    NavigationPage,
    ProductRowWidget,
    ProductRowType
};

use std::rc::{Rc, Weak};

struct State {
    ui: Ui,
    repository: Repository
}

struct WeakProductActivation {
    state: Weak<State>
}

impl WeakProductActivation {
    fn upgrade(&self) -> Option<ProductActivation> {
        self.state.upgrade().map(|state| ProductActivation { state })
    }
}

pub struct ProductActivation {
    state: Rc<State>
}

impl ProductActivation {

    pub fn new(ui: Ui, repository: Repository) -> Self {
        let state = State { ui, repository };
        let this = Self { state: Rc::new(state) };
        this.setup_rows_activation();
        this
    }

    fn setup_rows_activation(&self) {
        for (_, group) in self.state.ui.overview_page().groups().iter() {
            for row in group.rows().values() {
                let this_weak = self.downgrade();
                row.connect_activated(move |row| {
                    if let Some(this) = this_weak.upgrade() {
                        this.navigate_to_product_page(row);
                    }
                });
            }
        }
    }

    fn navigate_to_product_page(&self, row: &OverviewProductRowWidget) {
        let product_index = row.index() as usize;
        if let Some(product) = self.state.repository.product_by_index(product_index) {
            self.update_product_details(product_index, product);
            self.state.ui.navigation().push_page(NavigationPage::Product);
        }
    }

    fn update_product_details(&self, product_index: usize, product: &Product) {
        let product_page = self.state.ui.product_page();
        product_page.set_name(product.name);
        product_page.set_description(product.description);
        product_page.set_logo(product.logo);
        product_page.set_index(product_index as u32);
        product_page.remove_all_rows();

        if let Some(country) = product.country {
            let row = ProductRowWidget::from_country(country);
            product_page.append_row(row, ProductRowType::Country);
        }

        for (index, (property, website)) in product.websites.iter().enumerate() {
            let row = ProductRowWidget::from_website(property, website, index);
            product_page.append_row(row, ProductRowType::Website);
        }

        for &category_index in product.categories {
            if let Some(category) = self.state.repository.category_by_index(category_index) {
                let row = ProductRowWidget::from_category(category, category_index);
                product_page.append_row(row, ProductRowType::Category);
            }
        }
    }

    fn downgrade(&self) -> WeakProductActivation {
        let state = Rc::downgrade(&self.state);
        WeakProductActivation { state }
    }

}

