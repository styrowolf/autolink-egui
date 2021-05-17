use eframe::{egui, epi};
use std::sync::{Arc, Mutex};
use autolink_lib::{Plan, TimeDay};
use crossbeam_channel;
use autolink_lib::chrono;
use chrono::{Datelike, Timelike};
use std::iter::FromIterator;

mod utils;
mod state;

use state::{UIState, AddUIState, EditUIState, RemoveUIState};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct App {
    plans: Arc<Mutex<Vec<Plan>>>,
    is_loop_running: bool,
    sender: crossbeam_channel::Sender<bool>,
    receiver: crossbeam_channel::Receiver<bool>,
    state: UIState,
}



impl Default for App {
    fn default() -> Self {
        let (s, r) = crossbeam_channel::unbounded();
        Self {
            plans: Arc::new(Mutex::new(Vec::new())),
            is_loop_running: false,
            sender: s,
            receiver: r,
            state: UIState::default(),
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "autolink"
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::containers::CentralPanel::default().show(&ctx, |ui| {
            egui::containers::TopPanel::top("sections").show(&ctx, |ui| {
                ui.horizontal(|ui| {
                    /*
                    egui::containers::ComboBox::from_label("sections").show_index(ui, &mut self.state.section, 4, |i| {
                        String::from(["launch", "add", "edit", "remove"][i])
                    })
                    */
                    let scenes = ["launch", "add", "edit", "remove"];
                    ui.selectable_value(&mut self.state.section, 0, scenes[0]);
                    ui.selectable_value(&mut self.state.section, 1, scenes[1]);
                    ui.selectable_value(&mut self.state.section, 2, scenes[2]);
                    ui.selectable_value(&mut self.state.section, 3, scenes[3]);
                });
            });

            egui::containers::CentralPanel::default().show(&ctx, |ui| {
                if self.state.prev_section != self.state.section {
                    self.state.set_sections_to_default();
                    self.state.prev_section = self.state.section;
                }
                if self.state.section == 0 {
                    if self.plans.lock().unwrap().len() == 0 {
                        ui.add(egui::widgets::Label::new("first of all, add some entries").wrap(true));
                    } else {
                        ui.add(egui::widgets::Label::new("automagically launch entries at the right time").wrap(true));
                        if ui.button(match self.is_loop_running {
                            true => "stop loop",
                            false => "start loop"
                        }).clicked() {
                            match self.is_loop_running {
                                true => {
                                    let _ = self.sender.send(false);
                                    self.is_loop_running = false;
                                },
                                false => {
                                    let _ = self.sender.send(true);
                                    self.is_loop_running = true;
                                },
                            }
                        }
                        ui.add(egui::widgets::Separator::default().horizontal());
                        ui.add(egui::widgets::Label::new("manually select and launch an entry").wrap(true));
                        utils::entry_selection_box(ui, self.plans.clone(), &mut self.state.start.selection);
                        if ui.button("launch").clicked() {
                            let plans = self.plans.lock().unwrap().clone();
                            autolink_lib::open_link(
                                &plans.get(self.state.start.selection).unwrap().link
                            );
                        }
                    }
                    
                } else if self.state.section == 1 {
                    ui.add(egui::widgets::Label::new("name of entry").wrap(true));
                    ui.add(egui::widgets::TextEdit::singleline(&mut self.state.add.name).hint_text("name"));
                    ui.add(egui::widgets::Label::new("link of entry").wrap(true));
                    ui.add(egui::widgets::TextEdit::singleline(&mut self.state.add.link).hint_text("link"));
                    ui.add(egui::widgets::Separator::default().horizontal());
                    ui.add(egui::widgets::Checkbox::new(&mut self.state.add.add_time, "add time?"));
                    if self.state.add.add_time {
                        utils::day_selection_box(ui, &mut self.state.add.selected_day);
                        ui.horizontal( |ui| {
                            ui.add(egui::widgets::DragValue::new(&mut self.state.add.hour).clamp_range(0..=23));
                            ui.add(egui::widgets::Label::new("hour").wrap(true));
                            ui.add(egui::widgets::DragValue::new(&mut self.state.add.minute).clamp_range(0..=59));
                            ui.add(egui::widgets::Label::new("minute").wrap(true));
                        });
                    }
                    ui.add(egui::widgets::Separator::default().horizontal());
                    if ui.button("add").clicked() {
                        if !(self.state.add.name == "" || self.state.add.link == "") {
                            let mut plans = self.plans.lock().unwrap();
                            plans.push(Plan {
                                name: self.state.add.name.clone(),
                                link: self.state.add.link.clone(),
                                times: match self.state.add.add_time {
                                    true => { vec![
                                        TimeDay {
                                            day: utils::usize_to_day(self.state.add.selected_day),
                                            time: autolink_lib::chrono::NaiveTime::from_num_seconds_from_midnight((self.state.add.hour * 3600 + self.state.add.minute * 60) as u32, 0),
                                        }
                                    ] },
                                    false => { vec![] },
                                }
                            });
                            drop(plans);
                            let name = self.state.add.name.clone();
                            self.state.add = AddUIState::default();
                            self.state.add.output = format!("entry {} has been added", name);
                            let _ = self.sender.send(false);
                            let _ = self.sender.send(true);
                        } else {
                            self.state.add.output = String::from("both name and link must be entered!");
                        }
                    }
                    ui.add(egui::widgets::Label::new(self.state.add.output.clone()).wrap(true));
                } else if self.state.section == 2 {
                    if self.plans.lock().unwrap().len() == 0 {
                        ui.add(egui::widgets::Label::new("first of all, add some entries").wrap(true));
                    } else {
                        utils::entry_selection_box(ui, self.plans.clone(), &mut self.state.edit.selection);
                        ui.add(egui::widgets::Separator::default().horizontal());
                        if self.state.edit.selection != self.state.edit.prev_selection {
                            self.state.edit.prev_selection = self.state.edit.selection;
                            self.state.edit.plan = self.plans.lock().unwrap().get(self.state.edit.selection).unwrap().clone();
                        }
                        ui.add(egui::widgets::Label::new("name of entry").wrap(true));
                        ui.add(egui::widgets::TextEdit::singleline(&mut self.state.edit.plan.name).hint_text("name"));
                        ui.add(egui::widgets::Label::new("link of entry").wrap(true));
                        ui.add(egui::widgets::TextEdit::singleline(&mut self.state.edit.plan.link).hint_text("link"));
                        ui.add(egui::widgets::Separator::default().horizontal());
                        ui.add(egui::widgets::Checkbox::new(&mut self.state.edit.add_time, "add time?"));
                        if self.state.edit.add_time {
                            utils::day_selection_box(ui, &mut self.state.edit.selected_day);
                            ui.horizontal( |ui| {
                                ui.add(egui::widgets::DragValue::new(&mut self.state.edit.hour).clamp_range(0..=23));
                                ui.add(egui::widgets::Label::new("hour").wrap(true));
                                ui.add(egui::widgets::DragValue::new(&mut self.state.edit.minute).clamp_range(0..=59));
                                ui.add(egui::widgets::Label::new("minute").wrap(true));
                            });
                        }
                        ui.add(egui::widgets::Separator::default().horizontal());
                        egui::containers::Frame::default().show(ui, |ui| {
                            if self.state.edit.plan.times.len() != 0 {
                                ui.add(egui::widgets::Checkbox::new(&mut self.state.edit.remove_time, "remove time?"));
                                if self.state.edit.remove_time {
                                    let plan = self.state.edit.plan.clone();
                                    egui::containers::ComboBox::from_label("select time").show_index(ui, &mut self.state.edit.selected_time, self.state.edit.plan.times.len(), |i| {
                                        let TimeDay { day, time } = plan.times.get(i).unwrap();
                                        format!("{} - {}", day, time)
                                    });
                                }
                            } else {
                                ui.add(egui::widgets::Label::new("no times entered for this entry"));
                                ui.set_enabled(false);
                            }
                        
                        });
                        ui.add(egui::widgets::Separator::default().horizontal());
                        if ui.button("edit").clicked() {
                            if !(self.state.edit.plan.name == "" || self.state.edit.plan.link == "") {
                                if self.state.edit.add_time {
                                    self.state.edit.plan.times.push(TimeDay {
                                        day: utils::usize_to_day(self.state.edit.selected_day),
                                        time: autolink_lib::chrono::NaiveTime::from_num_seconds_from_midnight((self.state.edit.hour * 3600 + self.state.edit.minute * 60) as u32, 0),
                                    });
                                }
                                if self.state.edit.remove_time {
                                    self.state.edit.plan.times.remove(self.state.edit.selected_time);
                                }
                                let mut plans = self.plans.lock().unwrap();
                                plans.as_mut_slice()[self.state.edit.selection] = self.state.edit.plan.clone();
                                drop(plans);
                                let name = self.state.edit.plan.name.clone();
                                self.state.edit.refresh(0..self.plans.lock().unwrap().len());
                                self.state.edit.output = format!("entry {} has been edited", name);
                                let _ = self.sender.send(false);
                                let _ = self.sender.send(true);
                            } else {
                                self.state.edit.output = format!("both name and link must be entered!");
                            }
                        }
                        ui.add(egui::widgets::Label::new(self.state.edit.output.clone()).wrap(true));
                    }
                    
                } else if self.state.section == 3 {
                    if self.plans.lock().unwrap().len() == 0 {
                        ui.add(egui::widgets::Label::new("first of all, add some entries").wrap(true));
                    } else { 
                        utils::entry_selection_box(ui, self.plans.clone(), &mut self.state.remove.selection);
                        if ui.button("remove").clicked() {
                            let mut plans = self.plans.lock().unwrap();
                            let Plan {name, .. } = plans.remove(self.state.remove.selection);
                            self.state.remove = RemoveUIState::default();
                            self.state.remove.output = format!("entry {} has been removed", name);
                            let _ = self.sender.send(false);
                            let _ = self.sender.send(true);
                            ui.add(egui::widgets::Label::new(self.state.remove.output.clone()).wrap(true));
                        }
                    }

                }
            });
        });
    }

    fn setup(&mut self, _ctx: &egui::CtxRef) {
        let mut dir = home::home_dir().unwrap();
        dir.push(".autolink");
        let plans = autolink_lib::import(dir);
        self.plans = Arc::new(Mutex::new(plans));
        std::thread::spawn({
            let plans = self.plans.clone();
            let receiver = self.receiver.clone();
            move || { 
                let mut is_running = false;
                let mut p = Vec::new();
                let mut cp = Vec::new();
                
                loop {
                    while !receiver.is_empty() {
                        is_running = receiver.recv().unwrap();
                        if is_running {
                            let v = plans.lock().unwrap();
                            p = Vec::from_iter(v.clone());
                            drop(v); // explicitly dropping v preemptively, I'm using mutexes for the first time and I don't want to go crazy
                            cp = Vec::from_iter(p.clone());
                        }
                    }
                    if is_running {
                        let day = chrono::Local::now().naive_local().date().weekday();
                        let time = chrono::Local::now().naive_local().time();
                        let time = chrono::NaiveTime::from_hms(time.hour(), time.minute(), 0);
                        let timeday = TimeDay::new(time, day);
                        let length = cp.len();
                        for i in 0..length {
                            if autolink_lib::check(cp.get(i).unwrap(), &timeday) {
                                let mut p = cp.get(i).unwrap().clone();
                                p.remove_matching_time(&timeday);
                                cp.remove(i);    
                                cp.push(p);
                                break
                            }
                        }

                        let mut times = 0;
                        for p in &cp {
                            times += p.times.len();
                        }
                        if times == 0 {
                            cp = p.clone()
                        }
                        std::thread::sleep(std::time::Duration::new(5, 0));
                    }
                    std::thread::sleep(std::time::Duration::new(0, 0.25e4 as u32))
                }
            }
        });
    }

    fn on_exit(&mut self) {
        let mut dir = home::home_dir().unwrap();
        dir.push(".autolink");
        let plans = self.plans.lock().unwrap().clone();
        autolink_lib::export(plans, dir);
    }
}



