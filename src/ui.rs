use super::widgets::{
    NavigationPage,
    WindowWidget,
    NavigationWidget,
    MainPageWidget,
    OverviewPageWidget,
    ProductPageWidget,
    SidebarWidget,
    SidebarCountryRowWidget,
    SidebarCategoryListWidget,
    SidebarSearchRowWidget
};

use std::rc::{Rc, Weak};

pub struct UiWeak {
    window: Weak<WindowWidget>
}

impl UiWeak {
    pub fn upgrade(&self) -> Option<Ui> {
        self.window.upgrade().map(|window| Ui { window })
    }
}

#[derive(Clone)]
pub struct Ui {
    window: Rc<WindowWidget>
}

impl Ui {

    pub fn new(window: WindowWidget) -> Self {
        Self { window: Rc::new(window) }
    }

    pub fn activate(&self) {
        self.window.navigation().replace_with_page(NavigationPage::Main);
    }

    pub fn window(&self) -> &WindowWidget {
        &self.window
    }

    pub fn navigation(&self) -> &NavigationWidget {
        self.window.navigation()
    }

    pub fn sidebar(&self) -> &SidebarWidget {
        self.navigation()
            .main_page()
            .sidebar()
    }

    pub fn main_page(&self) -> &MainPageWidget {
        self.navigation().main_page()
    }

    pub fn country_row(&self) -> &SidebarCountryRowWidget {
        self.navigation()
            .main_page()
            .sidebar()
            .primary_list()
            .country_row()
    }

    pub fn search_row(&self) -> &SidebarSearchRowWidget {
        self.navigation()
            .main_page()
            .sidebar()
            .primary_list()
            .search_row()
    }

    pub fn category_list(&self) -> &SidebarCategoryListWidget {
        self.navigation()
            .main_page()
            .sidebar()
            .category_list()
    }

    pub fn overview_page(&self) -> &OverviewPageWidget {
        self.navigation()
            .main_page()
            .overview()
    }

    pub fn product_page(&self) -> &ProductPageWidget {
        self.navigation()
            .product_page()
    }

    pub fn downgrade(&self) -> UiWeak {
        UiWeak { window: Rc::downgrade(&self.window) }
    }

}