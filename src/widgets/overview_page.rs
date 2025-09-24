use super::super::prelude::*;
use super::overview_product_group::OverviewProductGroup;
use super::page_content::PageContent;
use super::super::controllers::GroupScrollController;
use super::super::ordered_map::OrderedMap;

use std::cell::{Cell, Ref, RefCell, OnceCell};
use std::sync::OnceLock;

const ACTIVE_GROUP_CHANGED_SIGNAL: &str = "active-group-changed";

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/pt/timarques/european_choice/overview_page.ui")]
    #[properties(wrapper_type = super::OverviewPage)]
    pub struct OverviewPage {
        #[template_child(id = "overview-page-box")]
        pub box_container: TemplateChild<gtk::Box>,
        #[template_child(id = "overview-page-content")]
        pub content: TemplateChild<PageContent>,

        #[property(get, set)]
        pub subtitle: RefCell<String>,

        pub groups: RefCell<OrderedMap<OverviewProductGroup>>,
        pub active_index: Cell<Option<usize>>,
        pub scroll_controller: OnceCell<GroupScrollController>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OverviewPage {
        const NAME: &'static str = "OverviewPage";
        type Type = super::OverviewPage;
        type ParentType = adw::NavigationPage;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for OverviewPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_scroll_controller();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<[glib::subclass::Signal; 1]> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                [
                    glib::subclass::Signal::builder(ACTIVE_GROUP_CHANGED_SIGNAL).param_types([OverviewProductGroup::static_type()]).build()
                ]
            })
        }
    }

    impl WidgetImpl for OverviewPage {}
    impl NavigationPageImpl for OverviewPage {}
}

glib::wrapper! {
    pub struct OverviewPage(ObjectSubclass<imp::OverviewPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl OverviewPage {

    fn setup_scroll_controller(&self) {
        let this_weak = self.downgrade();
        let this = self.clone();
        let scrolled_window = self.imp().content.scrolled_window().clone();
        let box_container = self.imp().box_container.clone();
        let handler = move |index| {
            if let Some(this) = this_weak.upgrade() {
                this.set_active_group_index(index);
            }
        };
        let controller = GroupScrollController::new(
            this,
            box_container,
            scrolled_window,
            handler
        );
        self.imp().scroll_controller.set(controller).ok().expect("controller set once");
    }

    pub fn add_group(&self, group: OverviewProductGroup) -> usize {
        let index = group.index() as usize;
        let imp = self.imp();
        imp.box_container.append(&group);

        let mut map = imp.groups.borrow_mut();
        let was_empty = map.is_empty();
        map.insert(index, group);
        drop(map);

        if was_empty {
            self.set_active_group_index(index);
        }

        index
    }

    pub fn active_group(&self) -> Option<Ref<'_, OverviewProductGroup>> {
        self
            .imp()
            .active_index
            .get()
            .map_or_else(
                || None,
                |index| Ref::filter_map(self.groups(),|m| m.get(index)).ok()
            )
    }

    pub fn active_group_index(&self) -> Option<usize> {
        self.imp().active_index.get()
    }

    pub fn groups(&self) -> Ref<'_, OrderedMap<OverviewProductGroup>> {
        self.imp().groups.borrow()
    }

    fn set_active_group_index(&self, index: usize) -> bool {
        let imp = self.imp();

        if
            imp.active_index.get() != Some(index)
            && let Some(group) = self.groups().get(index)
        {
            imp.active_index.set(Some(index));
            self.set_subtitle(group.title());
            self.emit_by_name::<()>(ACTIVE_GROUP_CHANGED_SIGNAL, &[group]);

            true
        } else {
            false
        }
    }

    pub fn get_active_group(&self) -> Option<Ref<'_, OverviewProductGroup>> {
        self
            .imp()
            .active_index
            .get()
            .map_or_else(
                || None,
                |index| Ref::filter_map(self.groups(), |m| m.get(index)).ok()
            )
    }

    pub fn scroll_to_group_index(&self, index: usize) -> bool {
        self.imp()
            .scroll_controller
            .get()
            .unwrap()
            .scroll_to(index)
    }

    pub fn scroll_to_top(&self) -> bool {
        self.imp()
            .scroll_controller
            .get()
            .unwrap()
            .scroll_to_top()
    }

    pub fn connect_active_group_changed<F>(&self, callback: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, &OverviewProductGroup) + 'static,
    {
        self.connect_local(ACTIVE_GROUP_CHANGED_SIGNAL, false, move |values| {
            let this = values[0].get::<Self>().unwrap();
            let group = values[1].get::<&OverviewProductGroup>().unwrap();
            callback(&this, group);
            None
        })
    }
}
