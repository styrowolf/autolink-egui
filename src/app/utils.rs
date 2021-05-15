use eframe::egui;
use eframe::egui::Ui;
use crate::app::state::UIState;
use autolink_lib::{Plan, chrono};
use std::sync::{Arc, Mutex};

pub fn entry_selection_box(ui: &mut Ui, plans: Arc<Mutex<Vec<Plan>>>, selected: &mut usize) {
    let plans = plans.lock().unwrap().clone();
    egui::containers::ComboBox::from_label("select entry").show_index(ui, selected, plans.len(), |i| {
        plans.get(i).unwrap().name.clone()
    });
}

pub fn day_selection_box(ui: &mut Ui, selected_day: &mut usize) {
    let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
    egui::containers::ComboBox::from_label("select day").show_index(ui, selected_day, days.len(), |i| {
        String::from(days[i])
    });
}

pub fn usize_to_day(i: usize) -> chrono::Weekday {
    match i {
        0 => chrono::Weekday::Mon,
        1 => chrono::Weekday::Tue,
        2 => chrono::Weekday::Wed,
        3 => chrono::Weekday::Thu,
        4 => chrono::Weekday::Fri,
        5 => chrono::Weekday::Sat,
        6 => chrono::Weekday::Sun,
        _ => unreachable!(),
    }
} 