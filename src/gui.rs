use crate::domain::{Entry, Shift, Responsible};
use crate::storage::Store;
use iced::widget::{
    button, column, combo_box, container, row, scrollable, text, text_input, Space,
};
use iced::{Element, Length, Task, Theme};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn run(db: PathBuf) {
    let boot = move || App::new(db.clone());
    let _ = iced::application(boot, App::update, App::view).run();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Aktuell,
    Planbaustellen,
    Archiv,
    Ereignislog,
    Stammdaten,
    Uebergabe,
    Einstellungen,
}

pub struct App {
    db_path: PathBuf,
    store: Option<Store>,
    tab: Tab,
    // data
    aktuell_entries: Vec<Entry>,
    plan_entries: Vec<Entry>,
    archive_entries: Vec<Entry>,
    events: Vec<crate::domain::Event>,
    plants: Vec<String>,
    responsible: Vec<String>,
    responsible_full: Vec<Responsible>,
    statuses: Vec<(String, i32)>,
    gewerke: Vec<String>,
    current_user: String,
    dark_mode: bool,
    last_applied_dark: Option<bool>,
    status_msg: String,
    // filters
    filter_date: String,
    filter_shift: String,
    // new entry form
    new_date: String,
    new_time: String,
    new_shift: String,
    new_plant: String,
    new_status: String,
    new_sap: String,
    new_fault: String,
    new_responsible: String,
    new_gewerk: String,
    new_priority: String,
    new_action: String,
    new_downtime: String,
    new_images: String,
    // edit
    edit_id: Option<String>,
    edit_status: String,
    edit_time: String,
    edit_shift: String,
    edit_plant: String,
    edit_responsible: String,
    edit_gewerk: String,
    edit_sap: String,
    edit_fault: String,
    edit_action: String,
    edit_priority: String,
    edit_downtime: String,
    edit_handed: String,
    edit_images: String,
    // handover
    ho_date: String,
    ho_transition: String,
    ho_user: String,
    ho_taken_by: String,
    // stammdaten
    new_plant_name: String,
    new_resp_name: String,
    new_status_name: String,
    new_status_weight: String,
    new_gewerk_name: String,
    stamm_password: String,
    stamm_input: String,
    stamm_unlocked: bool,
    // settings
    db_path_str: String,
    // detail
    detail_id: Option<String>,
    // plan status edits
    plan_edits: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    Refresh,
    // filters
    FilterDateChanged(String),
    FilterShiftChanged(String),
    // new entry
    NewDateChanged(String),
    NewTimeChanged(String),
    NewShiftChanged(String),
    NewPlantChanged(String),
    NewStatusChanged(String),
    NewSapChanged(String),
    NewFaultChanged(String),
    NewResponsibleChanged(String),
    NewGewerkChanged(String),
    NewPriorityChanged(String),
    NewDowntimeChanged(String),
    NewActionChanged(String),
    NewImagesChanged(String),
    AddEntry,
    // edit
    EditEntry(String),
    EditStatusChanged(String),
    EditTimeChanged(String),
    EditShiftChanged(String),
    EditPlantChanged(String),
    EditResponsibleChanged(String),
    EditGewerkChanged(String),
    EditSapChanged(String),
    EditFaultChanged(String),
    EditActionChanged(String),
    EditPriorityChanged(String),
    EditDowntimeChanged(String),
    EditHandedChanged(String),
    EditImagesChanged(String),
    SaveEdit,
    CancelEdit,
    // details
    ShowDetail(String),
    CloseDetail,
    // handover
    HoDateChanged(String),
    HoTransitionChanged(String),
    HoUserChanged(String),
    HoTakenByChanged(String),
    DoHandover,
    // stammdaten
    StammPasswordInput(String),
    StammUnlock,
    StammLock,
    NewPlantNameChanged(String),
    AddPlant,
    DeletePlant(String),
    NewRespNameChanged(String),
    AddResponsible,
    DeleteResponsible(String),
    NewStatusNameChanged(String),
    NewStatusWeightChanged(String),
    AddStatus,
    DeleteStatus(String),
    NewGewerkNameChanged(String),
    AddGewerk,
    DeleteGewerk(String),
    // settings
    DbPathChanged(String),
    OpenDb,
    BrowseDb,
    ToggleTheme,
    // plan
    PlanStatusChanged(String, String),
    SavePlanStatus(String),
    // misc
    SetStatusMsg(String),
}

impl App {
    fn new(db: PathBuf) -> Self {
        let store = Store::open(&db).ok();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut app = Self {
            db_path: db.clone(),
            store,
            tab: Tab::Aktuell,
            aktuell_entries: vec![],
            plan_entries: vec![],
            archive_entries: vec![],
            events: vec![],
            plants: vec![],
            responsible: vec![],
            responsible_full: vec![],
            statuses: vec![],
            gewerke: vec![],
            current_user: String::new(),
            dark_mode: true,
            last_applied_dark: None,
            status_msg: String::new(),
            filter_date: today.clone(),
            filter_shift: "Früh".to_string(),
            new_date: today.clone(),
            new_time: chrono::Local::now().format("%H:%M").to_string(),
            new_shift: "Früh".to_string(),
            new_plant: String::new(),
            new_status: "offen".to_string(),
            new_sap: String::new(),
            new_fault: String::new(),
            new_responsible: String::new(),
            new_gewerk: String::new(),
            new_priority: String::new(),
            new_action: String::new(),
            new_downtime: String::new(),
            new_images: String::new(),
            edit_id: None,
            edit_status: String::new(),
            edit_time: String::new(),
            edit_shift: String::new(),
            edit_plant: String::new(),
            edit_responsible: String::new(),
            edit_gewerk: String::new(),
            edit_sap: String::new(),
            edit_fault: String::new(),
            edit_action: String::new(),
            edit_priority: String::new(),
            edit_downtime: String::new(),
            edit_handed: String::new(),
            edit_images: String::new(),
            ho_date: today.clone(),
            ho_transition: "Früh -> Spät".to_string(),
            ho_user: String::new(),
            ho_taken_by: String::new(),
            new_plant_name: String::new(),
            new_resp_name: String::new(),
            new_status_name: String::new(),
            new_status_weight: String::new(),
            new_gewerk_name: String::new(),
            stamm_password: String::new(),
            stamm_input: String::new(),
            stamm_unlocked: false,
            db_path_str: db.to_string_lossy().to_string(),
            detail_id: None,
            plan_edits: HashMap::new(),
        };
        app.reload();
        app
    }

    fn reload(&mut self) {
        let Some(store) = self.store.as_ref() else { return; };
        self.plants = store.list_plants().unwrap_or_default();
        self.responsible = store.list_responsible().unwrap_or_default();
        self.responsible_full = store.list_responsible_full().unwrap_or_default();
        self.statuses = store.list_statuses().unwrap_or_default();
        self.gewerke = store.list_gewerke().unwrap_or_default();
        if self.current_user.is_empty() && !self.responsible.is_empty() {
            self.current_user = self.responsible[0].clone();
        }
        let t = store.get_setting("theme").unwrap_or_default();
        self.dark_mode = t != "light";
        if store.get_setting("stammdaten_password").unwrap_or_default().is_empty() {
            let _ = store.set_setting("stammdaten_password", "0000");
        }
        self.aktuell_entries = store
            .list_entries(false)
            .unwrap_or_default()
            .into_iter()
            .filter(|e| e.status != "geplant")
            .collect();
        self.plan_entries = store
            .list_entries(false)
            .unwrap_or_default()
            .into_iter()
            .filter(|e| e.status == "geplant")
            .collect();
        self.archive_entries = store.list_entries(true).unwrap_or_default();
        self.events = store.list_events(200).unwrap_or_default();
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(t) => {
                self.tab = t;
                Task::none()
            }
            Message::Refresh => {
                self.reload();
                Task::none()
            }
            Message::FilterDateChanged(v) => {
                self.filter_date = v;
                Task::none()
            }
            Message::FilterShiftChanged(v) => {
                self.filter_shift = v;
                Task::none()
            }
            Message::NewDateChanged(v) => {
                self.new_date = v;
                Task::none()
            }
            Message::NewTimeChanged(v) => {
                self.new_time = v;
                Task::none()
            }
            Message::NewShiftChanged(v) => {
                self.new_shift = v;
                Task::none()
            }
            Message::NewPlantChanged(v) => {
                self.new_plant = v;
                Task::none()
            }
            Message::NewStatusChanged(v) => {
                self.new_status = v;
                Task::none()
            }
            Message::NewSapChanged(v) => {
                self.new_sap = v;
                Task::none()
            }
            Message::NewFaultChanged(v) => {
                self.new_fault = v;
                Task::none()
            }
            Message::NewResponsibleChanged(v) => {
                self.new_responsible = v;
                Task::none()
            }
            Message::NewGewerkChanged(v) => {
                self.new_gewerk = v;
                Task::none()
            }
            Message::NewPriorityChanged(v) => {
                self.new_priority = v;
                Task::none()
            }
            Message::NewDowntimeChanged(v) => {
                self.new_downtime = v;
                Task::none()
            }
            Message::NewActionChanged(v) => {
                self.new_action = v;
                Task::none()
            }
            Message::NewImagesChanged(v) => {
                self.new_images = v;
                Task::none()
            }
            Message::AddEntry => {
                if let Some(store) = self.store.as_mut() {
                    let date = self.new_date.parse().unwrap_or_else(|_| chrono::Local::now().date_naive());
                    let mut e = Entry::new(date, Shift::from_str(&self.new_shift));
                    e.time = if self.new_time.trim().is_empty() {
                        chrono::Local::now().format("%H:%M").to_string()
                    } else {
                        self.new_time.clone()
                    };
                    e.status = self.new_status.clone();
                    e.plant = self.new_plant.clone();
                    e.sap_order = self.new_sap.clone();
                    e.fault = self.new_fault.clone();
                    e.responsible = self.new_responsible.clone();
                    e.gewerk = self.new_gewerk.clone();
                    e.priority = self.new_priority.trim().parse().ok();
                    e.downtime = self.new_downtime.clone();
                    e.action = self.new_action.clone();
                    e.images = self.new_images.clone();
                    e.last_editor = self.current_user.clone();
                    let _ = store.insert_entry(&e);
                    let _ = store.log_event("Neuer Eintrag", &self.current_user, &format!("{} - {}", e.plant, e.fault), &e.id);
                    self.new_plant.clear();
                    self.new_fault.clear();
                    self.new_sap.clear();
                    self.new_action.clear();
                    self.new_images.clear();
                    self.reload();
                    self.status_msg = "Eintrag hinzugefügt".to_string();
                }
                Task::none()
            }
            Message::EditEntry(id) => {
                if let Some(e) = self.aktuell_entries.iter().find(|x| x.id == id).cloned() {
                    self.edit_id = Some(id);
                    self.edit_status = e.status.clone();
                    self.edit_time = e.time.clone();
                    self.edit_shift = e.shift.as_str().to_string();
                    self.edit_plant = e.plant.clone();
                    self.edit_responsible = e.responsible.clone();
                    self.edit_gewerk = e.gewerk.clone();
                    self.edit_sap = e.sap_order.clone();
                    self.edit_fault = e.fault.clone();
                    self.edit_action = e.action.clone();
                    self.edit_priority = e.priority.map(|p| p.to_string()).unwrap_or_default();
                    self.edit_downtime = e.downtime.clone();
                    self.edit_handed = e.handed_over_to.clone();
                    self.edit_images = e.images.clone();
                }
                Task::none()
            }
            Message::EditStatusChanged(v) => {
                self.edit_status = v;
                Task::none()
            }
            Message::EditTimeChanged(v) => {
                self.edit_time = v;
                Task::none()
            }
            Message::EditShiftChanged(v) => {
                self.edit_shift = v;
                Task::none()
            }
            Message::EditPlantChanged(v) => {
                self.edit_plant = v;
                Task::none()
            }
            Message::EditResponsibleChanged(v) => {
                self.edit_responsible = v;
                Task::none()
            }
            Message::EditGewerkChanged(v) => {
                self.edit_gewerk = v;
                Task::none()
            }
            Message::EditSapChanged(v) => {
                self.edit_sap = v;
                Task::none()
            }
            Message::EditFaultChanged(v) => {
                self.edit_fault = v;
                Task::none()
            }
            Message::EditActionChanged(v) => {
                self.edit_action = v;
                Task::none()
            }
            Message::EditPriorityChanged(v) => {
                self.edit_priority = v;
                Task::none()
            }
            Message::EditDowntimeChanged(v) => {
                self.edit_downtime = v;
                Task::none()
            }
            Message::EditHandedChanged(v) => {
                self.edit_handed = v;
                Task::none()
            }
            Message::EditImagesChanged(v) => {
                self.edit_images = v;
                Task::none()
            }
            Message::SaveEdit => {
                if let Some(id) = self.edit_id.clone() {
                    if let Some(store) = self.store.as_mut() {
                        if let Some(mut e) = store.get_entry(&id).unwrap_or(None) {
                            e.status = self.edit_status.clone();
                            e.time = self.edit_time.clone();
                            e.shift = Shift::from_str(&self.edit_shift);
                            e.plant = self.edit_plant.clone();
                            e.responsible = self.edit_responsible.clone();
                            e.gewerk = self.edit_gewerk.clone();
                            e.sap_order = self.edit_sap.clone();
                            e.fault = self.edit_fault.clone();
                            e.action = self.edit_action.clone();
                            e.priority = self.edit_priority.trim().parse().ok();
                            e.downtime = self.edit_downtime.clone();
                            e.handed_over_to = self.edit_handed.clone();
                            e.images = self.edit_images.clone();
                            e.last_editor = self.current_user.clone();
                            e.updated_at = chrono::Local::now().naive_local();
                            let _ = store.update_entry(&e);
                            let _ = store.log_event("Eintrag bearbeitet", &self.current_user, &format!("{} - {}", e.plant, e.fault), &e.id);
                            self.status_msg = "Gespeichert".to_string();
                            self.reload();
                        }
                    }
                }
                Task::none()
            }
            Message::CancelEdit => {
                self.edit_id = None;
                Task::none()
            }
            Message::ShowDetail(id) => {
                self.detail_id = Some(id);
                Task::none()
            }
            Message::CloseDetail => {
                self.detail_id = None;
                Task::none()
            }
            Message::HoDateChanged(v) => {
                self.ho_date = v;
                Task::none()
            }
            Message::HoTransitionChanged(v) => {
                self.ho_transition = v;
                Task::none()
            }
            Message::HoUserChanged(v) => {
                self.ho_user = v;
                Task::none()
            }
            Message::HoTakenByChanged(v) => {
                self.ho_taken_by = v;
                Task::none()
            }
            Message::DoHandover => {
                if self.ho_user.trim().is_empty() {
                    self.ho_user = self.current_user.clone();
                }
                if self.ho_taken_by.trim().is_empty() {
                    self.ho_taken_by = self.current_user.clone();
                }
                if let Some(store) = self.store.as_mut() {
                    let date = self.ho_date.parse().unwrap_or_else(|_| chrono::Local::now().date_naive());
                    let (f, t) = match self.ho_transition.as_str() {
                        "Spät -> Nacht" => ("Spät", "Nacht"),
                        "Nacht -> Früh" => ("Nacht", "Früh"),
                        _ => ("Früh", "Spät"),
                    };
                    let from = Shift::from_str(f);
                    let to = Shift::from_str(t);
                    let archived = store.archive_completed(Some(date), Some(from)).unwrap_or(0);
                    let carried = store.carry_over_open(from, to, Some(date)).unwrap_or(0);
                    let h = crate::domain::Handover {
                        id: uuid::Uuid::new_v4().to_string(),
                        from_shift: from,
                        to_shift: to,
                        date,
                        performed_by: self.ho_user.clone(),
                        taken_over_by: self.ho_taken_by.clone(),
                        performed_at: chrono::Local::now().naive_local(),
                        archived_count: archived,
                        carried_count: carried,
                    };
                    let _ = store.record_handover(&h);
                    let _ = store.log_event(
                        "Schichtübergabe",
                        &self.ho_user,
                        &format!("von {} nach {} | übergeben von {} | übernommen von {}", f, t, self.ho_user, self.ho_taken_by),
                        &h.id,
                    );
                    self.status_msg = format!("Übergabe {} -> {}: archiviert={}, übernommen={}", f, t, archived, carried);
                    self.reload();
                }
                Task::none()
            }
            Message::StammPasswordInput(v) => {
                self.stamm_input = v;
                Task::none()
            }
            Message::StammUnlock => {
                let stored = self
                    .store
                    .as_ref()
                    .map(|st| st.get_setting("stammdaten_password").unwrap_or_default())
                    .unwrap_or_default();
                let stored = if stored.is_empty() {
                    "0000".to_string()
                } else {
                    stored
                };
                if self.stamm_input == stored {
                    self.stamm_unlocked = true;
                    self.stamm_input.clear();
                    self.status_msg = "Entsperrt".to_string();
                } else {
                    self.status_msg = "Falsches Passwort".to_string();
                }
                Task::none()
            }
            Message::StammLock => {
                self.stamm_unlocked = false;
                Task::none()
            }
            Message::NewPlantNameChanged(v) => {
                self.new_plant_name = v;
                Task::none()
            }
            Message::AddPlant => {
                if let Some(store) = self.store.as_mut() {
                    if !self.new_plant_name.trim().is_empty() {
                        let _ = store.add_plant(self.new_plant_name.trim());
                        self.new_plant_name.clear();
                        self.reload();
                    }
                }
                Task::none()
            }
            Message::DeletePlant(name) => {
                if let Some(store) = self.store.as_mut() {
                    let _ = store.delete_plant(&name);
                    self.reload();
                }
                Task::none()
            }
            Message::NewRespNameChanged(v) => {
                self.new_resp_name = v;
                Task::none()
            }
            Message::AddResponsible => {
                if let Some(store) = self.store.as_mut() {
                    if !self.new_resp_name.trim().is_empty() {
                        let _ = store.add_responsible(self.new_resp_name.trim());
                        self.new_resp_name.clear();
                        self.reload();
                    }
                }
                Task::none()
            }
            Message::DeleteResponsible(name) => {
                if let Some(store) = self.store.as_mut() {
                    let _ = store.delete_responsible(&name);
                    self.reload();
                }
                Task::none()
            }
            Message::NewStatusNameChanged(v) => {
                self.new_status_name = v;
                Task::none()
            }
            Message::NewStatusWeightChanged(v) => {
                self.new_status_weight = v;
                Task::none()
            }
            Message::AddStatus => {
                if let Some(store) = self.store.as_mut() {
                    if !self.new_status_name.trim().is_empty() {
                        let w: i32 = self.new_status_weight.trim().parse().unwrap_or(0);
                        let _ = store.add_status(self.new_status_name.trim(), w);
                        self.new_status_name.clear();
                        self.new_status_weight.clear();
                        self.reload();
                    }
                }
                Task::none()
            }
            Message::DeleteStatus(name) => {
                if let Some(store) = self.store.as_mut() {
                    let _ = store.delete_status(&name);
                    self.reload();
                }
                Task::none()
            }
            Message::NewGewerkNameChanged(v) => {
                self.new_gewerk_name = v;
                Task::none()
            }
            Message::AddGewerk => {
                if let Some(store) = self.store.as_mut() {
                    if !self.new_gewerk_name.trim().is_empty() {
                        let _ = store.add_gewerk(self.new_gewerk_name.trim());
                        self.new_gewerk_name.clear();
                        self.reload();
                    }
                }
                Task::none()
            }
            Message::DeleteGewerk(name) => {
                if let Some(store) = self.store.as_mut() {
                    let _ = store.delete_gewerk(&name);
                    self.reload();
                }
                Task::none()
            }
            Message::DbPathChanged(v) => {
                self.db_path_str = v;
                Task::none()
            }
            Message::OpenDb => {
                let p = PathBuf::from(&self.db_path_str);
                if let Ok(st) = Store::open(&p) {
                    self.store = Some(st);
                    self.db_path = p;
                    self.reload();
                    self.status_msg = "DB geöffnet".to_string();
                } else {
                    self.status_msg = "Fehler beim Öffnen".to_string();
                }
                Task::none()
            }
            Message::BrowseDb => Task::none(),
            Message::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
                if let Some(st) = self.store.as_ref() {
                    let _ = st.set_setting(
                        "theme",
                        if self.dark_mode { "dark" } else { "light" },
                    );
                }
                Task::none()
            }
            Message::PlanStatusChanged(id, v) => {
                self.plan_edits.insert(id, v);
                Task::none()
            }
            Message::SavePlanStatus(id) => {
                if let Some(store) = self.store.as_mut() {
                    if let Some(ns) = self.plan_edits.get(&id).cloned() {
                        if let Some(mut e) = store.get_entry(&id).unwrap_or(None) {
                            e.status = ns;
                            e.updated_at = chrono::Local::now().naive_local();
                            let _ = store.update_entry(&e);
                            let _ = store.log_event(
                                "Status geändert",
                                &self.current_user,
                                &format!("{} - {}", e.plant, e.fault),
                                &e.id,
                            );
                            self.reload();
                        }
                    }
                }
                Task::none()
            }
            Message::CloseDetail => {
                self.detail_id = None;
                Task::none()
            }
            Message::SetStatusMsg(m) => {
                self.status_msg = m;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let title = text("Schichtbuch (iced)").size(24);
        let tabs = row![
            button("Aktuell").on_press(Message::TabSelected(Tab::Aktuell)),
            button("Planbaustellen").on_press(Message::TabSelected(Tab::Planbaustellen)),
            button("Archiv").on_press(Message::TabSelected(Tab::Archiv)),
            button("Ereignislog").on_press(Message::TabSelected(Tab::Ereignislog)),
            button("Stammdaten").on_press(Message::TabSelected(Tab::Stammdaten)),
            button("Übergabe").on_press(Message::TabSelected(Tab::Uebergabe)),
            button("Einstellungen").on_press(Message::TabSelected(Tab::Einstellungen)),
        ]
        .spacing(8);

        let content: Element<Message> = match self.tab {
            Tab::Aktuell => self.view_aktuell(),
            Tab::Planbaustellen => self.view_planbaustellen(),
            Tab::Archiv => self.view_archiv(),
            Tab::Ereignislog => self.view_events(),
            Tab::Stammdaten => self.view_stammdaten(),
            Tab::Uebergabe => self.view_uebergabe(),
            Tab::Einstellungen => self.view_settings(),
        };

        let col = column![title, tabs, content]
            .spacing(10)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill);

        container(col).into()
    }

    fn view_aktuell(&self) -> Element<'_, Message> {
        let mut col = column![].spacing(6);
        col = col.push(text(format!("{} Einträge", self.aktuell_entries.len())));

        for e in &self.aktuell_entries {
            let roww = row![
                text(e.date.to_string()).width(Length::Fixed(90.0)),
                text(&e.time).width(Length::Fixed(50.0)),
                text(&e.status).width(Length::Fixed(110.0)),
                text(&e.plant).width(Length::Fixed(130.0)),
                text(&e.responsible).width(Length::Fixed(120.0)),
                text(&e.gewerk).width(Length::Fixed(100.0)),
                text(&e.fault).width(Length::Fixed(200.0)),
                button("Bearb.").on_press(Message::EditEntry(e.id.clone())),
                button("Details").on_press(Message::ShowDetail(e.id.clone())),
            ]
            .spacing(6);
            col = col.push(roww);
        }

        // new entry form
        col = col.push(text("Neuer Eintrag").size(16));
        col = col.push(row![
            text_input("Datum", &self.new_date).on_input(Message::NewDateChanged).width(Length::Fixed(100.0)),
            text_input("Uhrzeit", &self.new_time).on_input(Message::NewTimeChanged).width(Length::Fixed(60.0)),
            text_input("Schicht", &self.new_shift).on_input(Message::NewShiftChanged).width(Length::Fixed(60.0)),
            text_input("Anlage", &self.new_plant).on_input(Message::NewPlantChanged).width(Length::Fixed(120.0)),
            text_input("Status", &self.new_status).on_input(Message::NewStatusChanged).width(Length::Fixed(100.0)),
        ].spacing(6));
        col = col.push(row![
            text_input("SAP", &self.new_sap).on_input(Message::NewSapChanged).width(Length::Fixed(100.0)),
            text_input("Störmeldung", &self.new_fault).on_input(Message::NewFaultChanged).width(Length::Fixed(160.0)),
            text_input("Verantwortlich", &self.new_responsible).on_input(Message::NewResponsibleChanged).width(Length::Fixed(120.0)),
            text_input("Gewerk", &self.new_gewerk).on_input(Message::NewGewerkChanged).width(Length::Fixed(100.0)),
            text_input("Prio", &self.new_priority).on_input(Message::NewPriorityChanged).width(Length::Fixed(40.0)),
        ].spacing(6));
        col = col.push(row![
            text_input("Ausfallzeit", &self.new_downtime).on_input(Message::NewDowntimeChanged).width(Length::Fixed(100.0)),
            text_input("Maßnahme", &self.new_action).on_input(Message::NewActionChanged).width(Length::Fixed(200.0)),
            text_input("Bilder", &self.new_images).on_input(Message::NewImagesChanged).width(Length::Fixed(160.0)),
            button("Hinzufügen").on_press(Message::AddEntry),
        ].spacing(6));

        // edit form (if selected)
        if let Some(id) = &self.edit_id {
            col = col.push(text(format!("Bearbeiten: {}", id)).size(16));
            col = col.push(row![
                text_input("Status", &self.edit_status).on_input(Message::EditStatusChanged).width(Length::Fixed(120.0)),
                text_input("Schicht", &self.edit_shift).on_input(Message::EditShiftChanged).width(Length::Fixed(60.0)),
                text_input("Anlage", &self.edit_plant).on_input(Message::EditPlantChanged).width(Length::Fixed(120.0)),
                text_input("Verantwortlich", &self.edit_responsible).on_input(Message::EditResponsibleChanged).width(Length::Fixed(120.0)),
                text_input("Gewerk", &self.edit_gewerk).on_input(Message::EditGewerkChanged).width(Length::Fixed(100.0)),
            ].spacing(6));
            col = col.push(row![
                text_input("SAP", &self.edit_sap).on_input(Message::EditSapChanged).width(Length::Fixed(100.0)),
                text_input("Störmeldung", &self.edit_fault).on_input(Message::EditFaultChanged).width(Length::Fixed(160.0)),
                text_input("Ausfallzeit", &self.edit_downtime).on_input(Message::EditDowntimeChanged).width(Length::Fixed(100.0)),
                text_input("Übergeben an", &self.edit_handed).on_input(Message::EditHandedChanged).width(Length::Fixed(120.0)),
                text_input("Prio", &self.edit_priority).on_input(Message::EditPriorityChanged).width(Length::Fixed(40.0)),
            ].spacing(6));
            col = col.push(row![
                text_input("Maßnahme", &self.edit_action).on_input(Message::EditActionChanged).width(Length::Fixed(260.0)),
                text_input("Bilder", &self.edit_images).on_input(Message::EditImagesChanged).width(Length::Fixed(160.0)),
                button("Speichern").on_press(Message::SaveEdit),
                button("Schließen").on_press(Message::CancelEdit),
            ].spacing(6));
        }

        // detail popup (inline)
        if let Some(id) = &self.detail_id {
            if let Some(e) = self.aktuell_entries.iter().find(|x| x.id == *id) {
                col = col.push(text(format!("Details: {} - {}", e.plant, e.fault)).size(16));
                col = col.push(text(format!("Datum: {} {}", e.date, e.time)));
                col = col.push(text(format!("Status: {}", e.status)));
                col = col.push(text(format!("Verantwortlich: {}", e.responsible)));
                col = col.push(text(format!("Gewerk: {}", e.gewerk)));
                col = col.push(text(format!("SAP: {}", e.sap_order)));
                col = col.push(text(format!("Ausfallzeit: {}", e.downtime)));
                col = col.push(text(format!("Maßnahme: {}", e.action)));
                col = col.push(button("Schließen").on_press(Message::CloseDetail));
            }
        }

        if !self.status_msg.is_empty() {
            col = col.push(text(&self.status_msg));
        }

        scrollable(col).into()
    }

    fn view_planbaustellen(&self) -> Element<'_, Message> {
        let mut col = column![text("Planbaustellen").size(20)].spacing(8);
        for e in &self.plan_entries {
            col = col.push(row![
                text(e.date.to_string()).width(Length::Fixed(90.0)),
                text(&e.time).width(Length::Fixed(50.0)),
                text(&e.plant).width(Length::Fixed(130.0)),
                text(&e.fault).width(Length::Fixed(200.0)),
            ].spacing(6));
        }
        scrollable(col).into()
    }

    fn view_archiv(&self) -> Element<'_, Message> {
        let mut col = column![text("Archiv").size(20)].spacing(8);
        for e in &self.archive_entries {
            col = col.push(row![
                text(e.date.to_string()).width(Length::Fixed(90.0)),
                text(&e.plant).width(Length::Fixed(130.0)),
                text(&e.status).width(Length::Fixed(100.0)),
                text(&e.fault).width(Length::Fixed(200.0)),
            ].spacing(6));
        }
        scrollable(col).into()
    }

    fn view_events(&self) -> Element<'_, Message> {
        let mut col = column![text("Ereignislog").size(20)].spacing(8);
        for ev in &self.events {
            col = col.push(text(format!("{} | {} | {} | {}", ev.timestamp, ev.event_type, ev.user, ev.details)));
        }
        scrollable(col).into()
    }

    fn view_stammdaten(&self) -> Element<'_, Message> {
        if !self.stamm_unlocked {
            let mut col = column![text("Stammdaten passwortgeschützt").size(18)].spacing(8);
            col = col.push(row![
                text_input("Passwort", &self.stamm_input).on_input(Message::StammPasswordInput).width(Length::Fixed(120.0)),
                button("Entsperren").on_press(Message::StammUnlock),
            ].spacing(6));
            if !self.status_msg.is_empty() {
                col = col.push(text(&self.status_msg));
            }
            return scrollable(col).into();
        }
        let mut col = column![].spacing(8);
        col = col.push(text("Benutzer").size(18));
        for u in &self.responsible_full {
            col = col.push(text(format!("{} | {} | {} | {} | {}", u.name, u.amain, u.telefon, u.schicht_vk, u.gewerk)));
        }
        col = col.push(text("Anlagen").size(18));
        for p in &self.plants {
            col = col.push(text(p));
        }
        col = col.push(text("Gewerke").size(18));
        for g in &self.gewerke {
            col = col.push(text(g));
        }
        col = col.push(text("Stati").size(18));
        for (n, w) in &self.statuses {
            col = col.push(text(format!("{} (Gewicht: {})", n, w)));
        }
        if !self.status_msg.is_empty() {
            col = col.push(text(&self.status_msg));
        }
        scrollable(col).into()
    }

    fn view_uebergabe(&self) -> Element<'_, Message> {
        let mut col = column![text("Schichtübergabe").size(20)].spacing(8);
        col = col.push(row![
            text("Datum:").width(Length::Fixed(80.0)),
            text_input("", &self.ho_date).on_input(Message::HoDateChanged).width(Length::Fixed(120.0)),
        ].spacing(6));
        col = col.push(row![
            text("Übergabe:").width(Length::Fixed(80.0)),
            text_input("", &self.ho_transition).on_input(Message::HoTransitionChanged).width(Length::Fixed(120.0)),
        ].spacing(6));
        col = col.push(row![
            text("Übergeben von:").width(Length::Fixed(120.0)),
            text_input("", &self.ho_user).on_input(Message::HoUserChanged).width(Length::Fixed(160.0)),
        ].spacing(6));
        col = col.push(row![
            text("Übernommen von:").width(Length::Fixed(120.0)),
            text_input("", &self.ho_taken_by).on_input(Message::HoTakenByChanged).width(Length::Fixed(160.0)),
        ].spacing(6));
        col = col.push(button("Schichtübergabe durchführen").on_press(Message::DoHandover));
        if !self.status_msg.is_empty() {
            col = col.push(text(&self.status_msg));
        }
        scrollable(col).into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        let mut col = column![text("Einstellungen").size(20)].spacing(8);
        col = col.push(text("Datenbank:"));
        col = col.push(text_input("", &self.db_path_str).on_input(Message::DbPathChanged).width(Length::Fixed(300.0)));
        col = col.push(button("Öffnen").on_press(Message::OpenDb));
        col = col.push(row![
            button(if self.dark_mode { "☀ Hell" } else { "🌙 Dunkel" }).on_press(Message::ToggleTheme),
        ]);
        if !self.status_msg.is_empty() {
            col = col.push(text(&self.status_msg));
        }
        scrollable(col).into()
    }
}
