use crate::prelude::*;
use super::sidebar_search_row::{SidebarSearchRow, SidebarSearchRowState};
use super::sidebar_country_row::{SidebarCountryRow, SidebarCountryRowState};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/pt/timarques/european_choice/sidebar_primary_list.ui")]
    pub struct SidebarPrimaryList {
        #[template_child(id = "sidebar-primary-list-box")]
        pub list: TemplateChild<gtk::ListBox>,
        #[template_child(id = "sidebar-primary-search-row")]
        pub search_row: TemplateChild<SidebarSearchRow>,
        #[template_child(id = "sidebar-primary-country-row")]
        pub country_row: TemplateChild<SidebarCountryRow>,
        #[template_child(id = "sidebar-primary-event-controller-focus")]
        pub event_controller_focus: TemplateChild<gtk::EventControllerFocus>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarPrimaryList {
        const NAME: &'static str = "SidebarPrimaryList";
        type Type = super::SidebarPrimaryList;
        type ParentType = adw::Bin;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    impl ObjectImpl for SidebarPrimaryList {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_search_row();
            self.obj().setup_country_row();
            self.obj().setup_row_selection();
            self.obj().setup_focus_handling();
        }
    }
    
    impl WidgetImpl for SidebarPrimaryList {}
    impl BinImpl for SidebarPrimaryList {}
}

glib::wrapper! {
    pub struct SidebarPrimaryList(ObjectSubclass<imp::SidebarPrimaryList>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl SidebarPrimaryList {

    fn setup_search_row(&self) {
        let this_weak = self.downgrade();
        self.imp().search_row.connect_state_changed(move |row, state| {
            if let Some(this) = this_weak.upgrade() {
                match state {
                    SidebarSearchRowState::Active => this.imp().list.select_row(Some(row)),
                    SidebarSearchRowState::Inactive => this.deactivate_search_row(),
                    SidebarSearchRowState::Idle => (),
                }
            }
        });
    }

    fn setup_country_row(&self) {
        let this_weak = self.downgrade();
        self.imp().country_row.connect_state_changed(move |row, state| {
            if let Some(this) = this_weak.upgrade() {
                match state {
                    SidebarCountryRowState::Active => this.imp().list.select_row(Some(row)),
                    SidebarCountryRowState::Inactive => this.deactivate_country_row(),
                }
            }
        });
    }

    fn setup_row_selection(&self) {
        let this_weak = self.downgrade();
        self.imp().list.connect_row_selected(move |_, row| {
            if let Some(this) = this_weak.upgrade() {
                match row {
                    Some(row) if row.downcast_ref::<SidebarSearchRow>().is_some() => {
                        if let Some(search_row) = row.downcast_ref::<SidebarSearchRow>() {
                            search_row.set_state(SidebarSearchRowState::Active);
                        }
                    },
                    Some(row) if row.downcast_ref::<SidebarCountryRow>().is_some() => {
                        if let Some(country_row) = row.downcast_ref::<SidebarCountryRow>() {
                            country_row.set_state(SidebarCountryRowState::Active);
                        }
                    },
                    _ => {
                        this.imp().search_row.set_state(SidebarSearchRowState::Inactive);
                        this.imp().country_row.set_state(SidebarCountryRowState::Inactive);
                    }
                }
            }
        });
    }

    fn setup_focus_handling(&self) {
        let this_weak = self.downgrade();
        self.imp().event_controller_focus.connect_leave(move |_| {
            if let Some(this) = this_weak.upgrade() {
                this.deactivate_search_row();
                this.deactivate_country_row();
            }
        });
    }

    fn deactivate_row(&self, row: &impl IsA<gtk::ListBoxRow>) {
        let imp = self.imp();
        imp.list.unselect_row(row);
        imp.list.grab_focus();
    }

    fn deactivate_search_row(&self) {
        self.deactivate_row(&*self.imp().search_row);
    }

    fn deactivate_country_row(&self) {
        self.deactivate_row(&*self.imp().country_row);
    }

    pub fn deactivate_rows(&self) {
        self.deactivate_search_row();
        self.deactivate_country_row();
    }

    pub fn search_row(&self) -> &SidebarSearchRow {
        &self.imp().search_row
    }

    pub fn country_row(&self) -> &SidebarCountryRow {
        &self.imp().country_row
    }
}