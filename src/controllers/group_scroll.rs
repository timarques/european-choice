use super::super::prelude::*;
use gtk::graphene::Point;
use std::cell::Cell;
use std::rc::{Rc, Weak};
use std::time::Duration;

use crate::widgets::{OverviewPageWidget, OverviewProductGroupWidget};

const SCROLL_DEBOUNCE: Duration = Duration::from_millis(100);
const ANIMATION_DURATION: Duration = Duration::from_millis(300);
const ANIMATION_FRAME_INTERVAL: Duration = Duration::from_millis(16);

struct State {
    overview_page: OverviewPageWidget,
    container_box: gtk::Box,
    scrolled_window: gtk::ScrolledWindow,
    previous_scroll_position: Cell<f64>,
    debounce_timeout: Cell<Option<(f64, glib::SourceId)>>,
    animation_timeout: Cell<Option<glib::SourceId>>,
    on_active_changed: Box<dyn Fn(usize) + 'static>,
}

struct WeakGroupScroll {
    state: Weak<State>,
}

impl WeakGroupScroll {
    fn upgrade(&self) -> Option<GroupScroll> {
        self.state.upgrade().map(|state| GroupScroll { state })
    }
}

pub struct GroupScroll {
    state: Rc<State>,
}

impl GroupScroll {
    pub fn new<F>(
        overview_page: OverviewPageWidget,
        container_box: gtk::Box,
        scrolled_window: gtk::ScrolledWindow,
        on_active_changed: F,
    ) -> Self
    where
        F: Fn(usize) + 'static,
    {
        let state = Rc::new(State {
            overview_page,
            scrolled_window,
            container_box,
            previous_scroll_position: Cell::new(0.0),
            debounce_timeout: Cell::new(None),
            animation_timeout: Cell::new(None),
            on_active_changed: Box::new(on_active_changed),
        });

        let this = Self { state };
        this.setup_scroll_change_handler();
        this.setup_scroll_key_handler();
        this
    }

    fn setup_scroll_change_handler(&self) {
        let this_weak = self.downgrade();
        self.state.scrolled_window.vadjustment().connect_value_changed(move |_| {
            if let Some(this) = this_weak.upgrade() {
                this.schedule_debounced_scroll_handler();
            }
        });
    }

    fn setup_scroll_key_handler(&self) {
        let this_weak = self.downgrade();
        self.state.scrolled_window.connect_scroll_child(move |_, scroll_type, horizontal| {
            this_weak
                .upgrade()
                .is_some_and(|this| this.handle_scroll_key_event(scroll_type, horizontal))
        });
    }

    fn schedule_debounced_scroll_handler(&self) {
        if let Some((_, id)) = self.state.debounce_timeout.take() {
            id.remove();
        }

        let this_weak = self.downgrade();
        let handler = move || {
            if let Some(this) = this_weak.upgrade() {
                this.handle_scroll_change();
            }
        };

        let timeout_id = glib::timeout_add_local_once(SCROLL_DEBOUNCE, handler);
        let current_scroll_position = self.state.scrolled_window.vadjustment().value();
        self.state.debounce_timeout.replace(Some((current_scroll_position, timeout_id)));
    }

    fn handle_scroll_key_event(&self, scroll_type: gtk::ScrollType, horizontal: bool) -> bool {
        match (horizontal, scroll_type) {
            (false, gtk::ScrollType::Start) => {
                self.scroll_to_top();
                true
            }
            (false, gtk::ScrollType::End) => {
                self.scroll_to_bottom();
                true
            }
            _ => false,
        }
    }

    pub fn scroll_to(&self, index: usize) -> bool {
        if 
            !self.is_current_active(index)
            && let Some(group) = self.state.overview_page.groups().get(index)
            && group.is_visible()
            && let Some((relative_top, _relative_bottom)) = self.calculate_group_viewport_bounds(group)
        {
            (self.state.on_active_changed)(index);
            let adjustment = self.state.scrolled_window.vadjustment();
            let target_position = adjustment.value() + relative_top;
            self.animate_scroll_to_position(target_position, Some(index));
            true
        } else {
            false
        }
    }

    pub fn scroll_to_top(&self) -> bool {
        self
            .find_first_visible_group_index()
            .is_some_and(|index| self.scroll_to(index))
    }

    pub fn scroll_to_bottom(&self) -> bool {
        self
            .find_last_visible_group_index()
            .is_some_and(|index| self.scroll_to(index))
    }

    fn animate_scroll_to_position(&self, target_position: f64, active_index: Option<usize>) {
        if let Some(timeout_id) = self.state.animation_timeout.take() {
            timeout_id.remove();
        }

        let adjustment = self.state.scrolled_window.vadjustment();
        let start_position = adjustment.value();
        let distance = target_position - start_position;

        if distance.abs() < 1.0 {
            if let Some(index) = active_index {
                (self.state.on_active_changed)(index);
            }
            return;
        }

        let animation_start_time = std::time::Instant::now();
        let this_weak = self.downgrade();

        let animation_callback = move || {
            this_weak.upgrade().map_or(glib::ControlFlow::Break, |this| {
                this.execute_animation_frame(animation_start_time, start_position, distance, active_index)
            })
        };

        let timeout_id = glib::timeout_add_local(ANIMATION_FRAME_INTERVAL, animation_callback);
        self.state.animation_timeout.set(Some(timeout_id));
    }

    fn execute_animation_frame(
        &self,
        start_time: std::time::Instant,
        start_position: f64,
        distance: f64,
        active_index: Option<usize>
    ) -> glib::ControlFlow {
        let elapsed = start_time.elapsed();
        let progress = (elapsed.as_millis() as f64 / ANIMATION_DURATION.as_millis() as f64).min(1.0);
        let eased_progress = 1.0 - (1.0 - progress).powi(3);
        let current_position = distance.mul_add(eased_progress, start_position);
        self.state.scrolled_window.vadjustment().set_value(current_position);

        if progress >= 1.0 {
            self.state.animation_timeout.set(None);
            if let Some(index) = active_index {
                (self.state.on_active_changed)(index);
            }
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    }

    fn find_first_visible_group_index(&self) -> Option<usize> {
        for (index, group) in self.state.overview_page.groups().iter() {
            if group.is_visible() {
                return Some(index);
            }
        }
        None
    }

    fn find_last_visible_group_index(&self) -> Option<usize> {
        let groups = self.state.overview_page.groups();
        for index in (0..groups.len()).rev() {
            if
                let Some(group) = groups.get_by_index(index)
                && group.is_visible()
            {
                return Some(group.index() as usize);
            }
        }
        None
    }

    fn is_current_active(&self, index: usize) -> bool {
        self.state
            .overview_page
            .get_active_group()
            .is_some_and(|g| g.index() as usize == index)
    }

    fn handle_scroll_change(&self) {
        let queued_scroll_position = self.state.debounce_timeout.take().map(|(position, _)| position);
        let adjustment = self.state.scrolled_window.vadjustment();
        let current_scroll_position = adjustment.value();
        let previous_scroll_position = self.state.previous_scroll_position.get();

        let effective_previous_position = queued_scroll_position
            .filter(|&queued| (queued - current_scroll_position).abs() < f64::EPSILON)
            .map(|_| previous_scroll_position)
            .or(queued_scroll_position)
            .unwrap_or(previous_scroll_position);

        self.state.previous_scroll_position.set(current_scroll_position);
        let scrolling_down = current_scroll_position > effective_previous_position;

        if 
            let Some(index) = self.find_active_group_by_viewport_intersection(scrolling_down)
            && !self.is_current_active(index)
        {
            (self.state.on_active_changed)(index);
        }
    }

    fn find_active_group_by_viewport_intersection(&self, scrolling_down: bool) -> Option<usize> {
        
        let adjustment = self.state.scrolled_window.vadjustment();
        let current_scroll_position = adjustment.value();
        let viewport_height = adjustment.page_size();
        let max_scroll_position = adjustment.upper() - viewport_height;

        if current_scroll_position <= 0.0 {
            return self.find_first_visible_group_index();
        }

        if current_scroll_position >= max_scroll_position {
            return self.find_last_visible_group_index();
        }

        let groups = self.state.overview_page.groups();
        let mut best_group_index = None;
        let mut best_intersection_score = f64::NEG_INFINITY;

        for (index, group) in groups.iter() {
            if 
                group.is_visible()
                && let Some((relative_top, relative_bottom)) = self.calculate_group_viewport_bounds(group)
                && relative_bottom > 0.0
                && relative_top < viewport_height
            {
                let intersection_score = if scrolling_down {
                    if relative_top <= 0.0 {
                        -relative_top
                    } else {
                        -relative_top - 1000.0
                    }
                } else {
                    -relative_top
                };

                if intersection_score > best_intersection_score {
                    best_intersection_score = intersection_score;
                    best_group_index = Some(index);
                }
            }
        }

        best_group_index
    }

    fn calculate_group_viewport_bounds(
        &self,
        group: &OverviewProductGroupWidget,
    ) -> Option<(f64, f64)> {
        let origin = Point::new(0.0, 0.0);
        group.compute_point(&self.state.container_box, &origin).map(|point_in_page| {
            let adjustment = self.state.scrolled_window.vadjustment();
            let scroll_position = adjustment.value();
            let group_y_position = f64::from(point_in_page.y());
            let group_height = f64::from(group.height());
            let relative_top = group_y_position - scroll_position;
            (relative_top, relative_top + group_height)
        })
    }

    fn downgrade(&self) -> WeakGroupScroll {
        let state = Rc::downgrade(&self.state);
        WeakGroupScroll { state }
    }
}