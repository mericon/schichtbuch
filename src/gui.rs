use crate::domain::{Entry, Shift, Status};
use crate::storage::Store;
use chrono::NaiveDate;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;

pub fn run(db: PathBuf) {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Schichtbuch",
        options,
        Box::new(move |_cc| {
            let mut app = SchichtbuchApp::new(db.clone());
            app.reload();
            Ok(Box::new(app))
        }),
    );
}

#[derive(PartialEq, Clone, Copy)]
enum Tab { Aktuell, Prio, Archiv, Frueh, Spaet, Nacht, Stammdaten, Uebergabe }

struct SchichtbuchApp {
    db_path: PathBuf,
    db_path_str: String,
    current_user: String,
    store: Option<Store>,
    tab: Tab,
    filter_date: String,
    filter_shift: String,
    sicht_date: String,
    current_entries: Vec<Entry>,
    prio_entries: Vec<Entry>,
    archive_query: String,
    archive_filter_plant: String,
    archive_results: Vec<Entry>,
    plants: Vec<String>,
    responsible: Vec<String>,
    selected_plant: Option<String>,
    new_plant_name: String,
    rename_plant_name: String,
    selected_resp: Option<String>,
    new_resp_name: String,
    rename_resp_name: String,
    // new entry form
    new_date: String,
    new_shift: String,
    new_plant: String,
    new_status: String,
    new_sap: String,
    new_fault: String,
    new_responsible: String,
    new_priority: String,
    new_action: String,
    new_downtime: String,
    new_images: String,
    // edit
    selected_id: Option<String>,
    detail_id: Option<String>,
    show_new_window: bool,
    show_edit_window: bool,
    edit_status: String,
    edit_shift: String,
    edit_plant: String,
    edit_responsible: String,
    edit_sap: String,
    edit_fault: String,
    edit_action: String,
    edit_priority: String,
    edit_downtime: String,
    edit_handed: String,
    edit_images: String,
    // handover
    ho_user: String,
    ho_from: String,
    ho_to: String,
    ho_date: String,
    status_msg: String,
}

impl SchichtbuchApp {
    fn new(db: PathBuf) -> Self {
        let store = Store::open(&db).ok();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        Self {
            db_path: db.clone(),
            db_path_str: db.to_string_lossy().to_string(),
            current_user: String::new(),
            store,
            tab: Tab::Aktuell,
            filter_date: today.clone(),
            filter_shift: "Früh".into(),
            sicht_date: today.clone(),
            current_entries: vec![],
            prio_entries: vec![],
            archive_query: String::new(),
            archive_filter_plant: String::new(),
            archive_results: vec![],
            plants: vec![],
            responsible: vec![],
            selected_plant: None,
            new_plant_name: String::new(),
            rename_plant_name: String::new(),
            selected_resp: None,
            new_resp_name: String::new(),
            rename_resp_name: String::new(),
            new_date: today.clone(),
            new_shift: "Früh".into(),
            new_plant: String::new(),
            new_status: "offen".into(),
            new_sap: String::new(),
            new_fault: String::new(),
            new_responsible: String::new(),
            new_priority: String::new(),
            new_action: String::new(),
            new_downtime: String::new(),
            new_images: String::new(),
            selected_id: None,
            detail_id: None,
            show_new_window: false,
            show_edit_window: false,
            edit_status: String::new(),
            edit_shift: String::new(),
            edit_plant: String::new(),
            edit_responsible: String::new(),
            edit_sap: String::new(),
            edit_fault: String::new(),
            edit_action: String::new(),
            edit_priority: String::new(),
            edit_downtime: String::new(),
            edit_handed: String::new(),
            edit_images: String::new(),
            ho_user: String::new(),
            ho_from: "Früh".into(),
            ho_to: "Spät".into(),
            ho_date: today,
            status_msg: String::new(),
        }
    }

    fn reload(&mut self) {
        let Some(store) = self.store.as_ref() else { return; };
        self.plants = store.list_plants().unwrap_or_default();
        self.responsible = store.list_responsible().unwrap_or_default();
        if self.current_user.is_empty() && !self.responsible.is_empty() {
            self.current_user = self.responsible[0].clone();
        }
        self.prio_entries = store.list_open_priorities().unwrap_or_default();
        if self.filter_date.trim().is_empty() { self.filter_date = chrono::Local::now().format("%Y-%m-%d").to_string(); }
        if let Ok(d) = self.filter_date.parse::<NaiveDate>() {
            self.current_entries = store.list_current_shift(d, Shift::from_str(&self.filter_shift)).unwrap_or_default();
            self.status_msg = format!("{} Einträge (aktuelle Schicht) | DB: {}", self.current_entries.len(), self.db_path.display());
        } else {
            self.status_msg = format!("Ungültiges Datum: {}", self.filter_date);
        }
        // archive: apply filter
        let mut res = if self.archive_query.trim().is_empty() {
            store.list_entries(true).unwrap_or_default()
        } else {
            store.search_archive(&self.archive_query).unwrap_or_default()
        };
        if !self.archive_filter_plant.trim().is_empty() {
            res.retain(|e| e.plant == self.archive_filter_plant);
        }
        self.archive_results = res;
    }

    fn load_edit(&mut self, e: &Entry) {
        self.selected_id = Some(e.id.clone());
        self.edit_status = e.status.as_str().to_string();
        self.edit_shift = e.shift.as_str().to_string();
        self.edit_plant = e.plant.clone();
        self.edit_responsible = e.responsible.clone();
        self.edit_sap = e.sap_order.clone();
        self.edit_fault = e.fault.clone();
        self.edit_action = e.action.clone();
        self.edit_priority = e.priority.map(|p| p.to_string()).unwrap_or_default();
        self.edit_downtime = e.downtime.clone();
        self.edit_handed = e.handed_over_to.clone();
        self.edit_images = e.images.clone();
    }

    fn save_edit(&mut self) {
        let Some(store) = self.store.as_mut() else { return; };
        let Some(id) = self.selected_id.clone() else { return; };
        if let Some(mut e) = store.get_entry(&id).unwrap_or(None) {
            e.status = Status::from_str(&self.edit_status);
            e.shift = Shift::from_str(&self.edit_shift);
            e.last_shift = Some(Shift::from_str(&self.filter_shift));
            e.last_editor = self.current_user.clone();
            e.plant = self.edit_plant.clone();
            e.responsible = self.edit_responsible.clone();
            e.sap_order = self.edit_sap.clone();
            e.fault = self.edit_fault.clone();
            e.action = self.edit_action.clone();
            e.priority = self.edit_priority.trim().parse().ok();
            e.downtime = self.edit_downtime.clone();
            e.handed_over_to = self.edit_handed.clone();
            e.images = self.edit_images.clone();
            e.updated_at = chrono::Local::now().naive_local();
            match store.update_entry(&e) {
                Ok(_) => { self.status_msg = "Gespeichert".into(); self.reload(); }
                Err(err) => self.status_msg = format!("Fehler: {}", err),
            }
        }
    }

    fn add_entry(&mut self) {
        let Some(store) = self.store.as_mut() else { return; };
        let date: NaiveDate = match self.new_date.parse() { Ok(d) => d, Err(_) => { self.status_msg="bad date".into(); return; } };
        let mut e = Entry::new(date, Shift::from_str(&self.new_shift));
        e.plant = self.new_plant.clone();
        e.status = Status::from_str(&self.new_status);
        e.last_shift = Some(Shift::from_str(&self.new_shift));
        e.last_editor = self.current_user.clone();
        e.sap_order = self.new_sap.clone();
        e.fault = self.new_fault.clone();
        e.responsible = self.new_responsible.clone();
        e.priority = self.new_priority.trim().parse().ok();
        e.action = self.new_action.clone();
        e.downtime = self.new_downtime.clone();
        e.images = self.new_images.clone();
        match store.insert_entry(&e) {
            Ok(_) => { self.status_msg = format!("added {}", e.id); self.filter_date = self.new_date.clone(); self.filter_shift = self.new_shift.clone(); self.reload(); }
            Err(err) => self.status_msg = format!("err: {}", err),
        }
    }

    fn do_handover(&mut self) {
        let Some(store) = self.store.as_mut() else { return; };
        let date: NaiveDate = match self.ho_date.parse() { Ok(d) => d, Err(_) => { self.status_msg="bad date".into(); return; } };
        let from = Shift::from_str(&self.ho_from);
        let to = Shift::from_str(&self.ho_to);
        let archived = store.archive_completed(Some(date), Some(from)).unwrap_or(0);
        let carried = store.list_open_priorities().unwrap_or_default().len() as i64;
        let h = crate::domain::Handover {
            id: uuid::Uuid::new_v4().to_string(),
            from_shift: from,
            to_shift: to,
            date,
            performed_by: self.ho_user.clone(),
            performed_at: chrono::Local::now().naive_local(),
            archived_count: archived,
            carried_count: carried,
        };
        match store.record_handover(&h) {
            Ok(_) => { self.status_msg = format!("Übergabe {} -> {}: archiviert={}, übernommen={}", self.ho_from, self.ho_to, archived, carried); self.reload(); }
            Err(e) => self.status_msg = format!("err: {}", e),
        }
    }
}

impl eframe::App for SchichtbuchApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Schichtbuch");
                ui.label("DB:");
                ui.text_edit_singleline(&mut self.db_path_str);
                if ui.button("Durchsuchen").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Datenbank", &["db", "sqlite", "sqlite3"])
                        .pick_file()
                    {
                        self.db_path_str = path.to_string_lossy().to_string();
                    }
                }
                if ui.button("Öffnen").clicked() {
                    let p = PathBuf::from(&self.db_path_str);
                    match Store::open(&p) {
                        Ok(st) => { self.store = Some(st); self.db_path = p; self.status_msg = "Datenbank geöffnet".into(); self.reload(); }
                        Err(e) => self.status_msg = format!("Fehler: {}", e),
                    }
                }
                ui.label("Bearbeiter:");
                egui::ComboBox::from_id_salt("user_sel").selected_text(if self.current_user.is_empty() {"(wählen)"} else {&self.current_user}).show_ui(ui, |ui| {
                    for r in &self.responsible.clone() { ui.selectable_value(&mut self.current_user, r.clone(), r); }
                });
                if ui.button("Aktualisieren").clicked() { self.reload(); }
            });
            if !self.status_msg.is_empty() {
                ui.label(&self.status_msg);
            }
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Aktuell, "Aktuell");
                ui.selectable_value(&mut self.tab, Tab::Prio, "Prio offen");
                ui.selectable_value(&mut self.tab, Tab::Archiv, "Archiv");
                ui.selectable_value(&mut self.tab, Tab::Frueh, "Früh");
                ui.selectable_value(&mut self.tab, Tab::Spaet, "Spät");
                ui.selectable_value(&mut self.tab, Tab::Nacht, "Nacht");
                ui.selectable_value(&mut self.tab, Tab::Stammdaten, "Stammdaten");
                ui.selectable_value(&mut self.tab, Tab::Uebergabe, "Übergabe");
            });
            ui.separator();
            match self.tab {
                Tab::Aktuell => self.ui_aktuell(ui),
                Tab::Prio => self.ui_prio(ui),
                Tab::Archiv => self.ui_archiv(ui),
                Tab::Frueh => self.ui_shift_view(ui, "Früh"),
                Tab::Spaet => self.ui_shift_view(ui, "Spät"),
                Tab::Nacht => self.ui_shift_view(ui, "Nacht"),
                Tab::Stammdaten => self.ui_stamm(ui),
                Tab::Uebergabe => self.ui_uebergabe(ui),
            }

            // Detail window
            if let Some(id) = self.detail_id.clone() {
                let ctx = ui.ctx().clone();
                let mut open = true;
                let entry = self.store.as_ref().and_then(|st| st.get_entry(&id).ok().flatten());
                egui::Window::new("Meldungs-Details").resizable(true).default_width(420.0).open(&mut open).show(&ctx, |ui| {
                    if let Some(e) = &entry {
                        TableBuilder::new(ui)
                            .resizable(true)
                            .column(Column::initial(130.0).resizable(true))
                            .column(Column::remainder().resizable(true))
                            .body(|mut body| {
                                let e = entry.as_ref().unwrap();
                                let rows: Vec<(&str, String)> = vec![
                                    ("Datum:", e.date.to_string()),
                                    ("Schicht:", e.shift.as_str().to_string()),
                                    ("Letzte Schicht:", e.last_shift.map(|s| s.as_str().to_string()).unwrap_or_default()),
                                    ("Letzter Bearbeiter:", e.last_editor.clone()),
                                    ("Status:", e.status.as_str().to_string()),
                                    ("Priorität:", e.priority.map(|p| p.to_string()).unwrap_or_default()),
                                    ("Anlage:", e.plant.clone()),
                                    ("SAP-Auftrag:", e.sap_order.clone()),
                                    ("Störmeldung:", e.fault.clone()),
                                    ("Verantwortlich:", e.responsible.clone()),
                                    ("An wen übergeben?", e.handed_over_to.clone()),
                                    ("Ausfallzeit:", e.downtime.clone()),
                                    ("Maßnahme:", e.action.clone()),
                                ];
                                for (k, v) in rows {
                                    body.row(16.0, |mut row| {
                                        row.col(|ui| { ui.strong(k); });
                                        row.col(|ui| { ui.label(v); });
                                    });
                                }
                                body.row(16.0, |mut row| {
                                    row.col(|ui| { ui.strong("Bilder:"); });
                                    row.col(|ui| {
                                        ui.vertical(|ui| {
                                            for img in e.images.split(';').filter(|s| !s.is_empty()) {
                                                let name = std::path::Path::new(img).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| img.to_string());
                                                if ui.link(&name).clicked() { let _ = opener::open(img); }
                                            }
                                            if e.images.is_empty() { ui.label("(keine)"); }
                                        });
                                    });
                                });
                            });
                    } else {
                        ui.label("Eintrag nicht gefunden");
                    }
                });
                if !open { self.detail_id = None; }
            }

            // New entry window
            if self.show_new_window {
                let ctx = ui.ctx().clone();
                let mut open = true;
                egui::Window::new("Neuer Eintrag").open(&mut open).show(&ctx, |ui| {
                    ui.horizontal(|ui| { ui.label("Datum"); ui.text_edit_singleline(&mut self.new_date); });
                    ui.horizontal(|ui| { ui.label("Schicht"); ui.text_edit_singleline(&mut self.new_shift); });
                    ui.horizontal(|ui| { ui.label("Anlage"); egui::ComboBox::from_id_salt("new_plant").selected_text(if self.new_plant.is_empty() {"(wählen)"} else {&self.new_plant}).show_ui(ui, |ui| {
                        for p in &self.plants.clone() { ui.selectable_value(&mut self.new_plant, p.clone(), p); }
                    }); });
                    ui.horizontal(|ui| { ui.label("Status"); egui::ComboBox::from_id_salt("new_status").selected_text(&self.new_status).show_ui(ui, |ui| {
                        for s in ["offen","In Bearbeitung","steht in Bearbeitung","erledigt"] { ui.selectable_value(&mut self.new_status, s.to_string(), s); }
                    }); });
                    ui.horizontal(|ui| { ui.label("SAP"); ui.text_edit_singleline(&mut self.new_sap); });
                    ui.horizontal(|ui| { ui.label("Störmeldung"); ui.text_edit_singleline(&mut self.new_fault); });
                    ui.horizontal(|ui| { ui.label("Verantwortlich"); egui::ComboBox::from_id_salt("new_resp").selected_text(if self.new_responsible.is_empty() {"(wählen)"} else {&self.new_responsible}).show_ui(ui, |ui| {
                        for r in &self.responsible.clone() { ui.selectable_value(&mut self.new_responsible, r.clone(), r); }
                    }); });
                    ui.horizontal(|ui| { ui.label("Prio"); ui.text_edit_singleline(&mut self.new_priority); });
                    ui.horizontal(|ui| { ui.label("Ausfallzeit"); ui.text_edit_singleline(&mut self.new_downtime); });
                    ui.horizontal(|ui| { ui.label("Maßnahme"); ui.text_edit_singleline(&mut self.new_action); });
                    ui.horizontal(|ui| { ui.label("Bilder"); ui.text_edit_singleline(&mut self.new_images);
                        if ui.button("Anhängen").clicked() {
                            if let Some(paths) = rfd::FileDialog::new().add_filter("Bilder", &["png","jpg","jpeg","bmp","gif"]).pick_files() {
                                for path in paths {
                                    if !self.new_images.is_empty() { self.new_images.push(';'); }
                                    self.new_images.push_str(&path.to_string_lossy());
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Hinzufügen").clicked() { self.add_entry(); self.show_new_window = false; }
                        if ui.button("Abbrechen").clicked() { self.show_new_window = false; }
                    });
                });
                if !open { self.show_new_window = false; }
            }

            // Edit window
            if self.show_edit_window && self.selected_id.is_some() {
                let ctx = ui.ctx().clone();
                let mut open = true;
                egui::Window::new("Eintrag bearbeiten").open(&mut open).show(&ctx, |ui| {
                    ui.horizontal(|ui| { ui.label("Status"); egui::ComboBox::from_id_salt("edit_status").selected_text(&self.edit_status).show_ui(ui, |ui| {
                        for s in ["offen","In Bearbeitung","steht in Bearbeitung","erledigt"] { ui.selectable_value(&mut self.edit_status, s.to_string(), s); }
                    }); });
                    ui.horizontal(|ui| { ui.label("Schicht"); egui::ComboBox::from_id_salt("edit_shift").selected_text(&self.edit_shift).show_ui(ui, |ui| {
                        for s in ["Früh","Spät","Nacht","Tag"] { ui.selectable_value(&mut self.edit_shift, s.to_string(), s); }
                    }); });
                    ui.horizontal(|ui| { ui.label("Anlage"); egui::ComboBox::from_id_salt("edit_plant").selected_text(if self.edit_plant.is_empty() {"(wählen)"} else {&self.edit_plant}).show_ui(ui, |ui| {
                        for p in &self.plants.clone() { ui.selectable_value(&mut self.edit_plant, p.clone(), p); }
                    }); });
                    ui.horizontal(|ui| { ui.label("Verantwortlich"); egui::ComboBox::from_id_salt("edit_resp").selected_text(if self.edit_responsible.is_empty() {"(wählen)"} else {&self.edit_responsible}).show_ui(ui, |ui| {
                        for r in &self.responsible.clone() { ui.selectable_value(&mut self.edit_responsible, r.clone(), r); }
                    }); });
                    ui.horizontal(|ui| { ui.label("SAP"); ui.text_edit_singleline(&mut self.edit_sap); });
                    ui.horizontal(|ui| { ui.label("Störmeldung"); ui.text_edit_singleline(&mut self.edit_fault); });
                    ui.horizontal(|ui| { ui.label("Ausfallzeit"); ui.text_edit_singleline(&mut self.edit_downtime); });
                    ui.horizontal(|ui| { ui.label("An wen übergeben?"); ui.text_edit_singleline(&mut self.edit_handed); });
                    ui.horizontal(|ui| { ui.label("Prio"); ui.text_edit_singleline(&mut self.edit_priority); });
                    ui.horizontal(|ui| { ui.label("Maßnahme"); ui.text_edit_singleline(&mut self.edit_action); });
                    ui.horizontal(|ui| { ui.label("Bilder"); ui.text_edit_singleline(&mut self.edit_images);
                        if ui.button("Anhängen").clicked() {
                            if let Some(paths) = rfd::FileDialog::new().add_filter("Bilder", &["png","jpg","jpeg","bmp","gif"]).pick_files() {
                                for path in paths {
                                    if !self.edit_images.is_empty() { self.edit_images.push(';'); }
                                    self.edit_images.push_str(&path.to_string_lossy());
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Speichern").clicked() { self.save_edit(); self.show_edit_window = false; }
                        if ui.button("Abbrechen").clicked() { self.show_edit_window = false; self.selected_id = None; }
                    });
                });
                if !open { self.show_edit_window = false; self.selected_id = None; }
            }
        });
    }
}

impl SchichtbuchApp {
    fn ui_aktuell(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Datum:"); ui.text_edit_singleline(&mut self.filter_date);
            ui.label("Schicht:"); ui.text_edit_singleline(&mut self.filter_shift);
            if ui.button("Laden").clicked() { self.reload(); }
        });
        ui.separator();
        if ui.button("Neuer Eintrag").clicked() { self.show_new_window = true; }
        ui.separator();

        // Table
        let mut clicked_id: Option<String> = None;
        let mut detail_id: Option<String> = None;
        let mut copied_sap: String = String::new();
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click())
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::initial(80.0).resizable(true))
            .column(Column::initial(90.0).resizable(true))
            .column(Column::initial(130.0).resizable(true))
            .column(Column::initial(80.0).resizable(true))
            .column(Column::initial(35.0).resizable(true))
            .column(Column::initial(60.0).resizable(true))
            .column(Column::initial(90.0).resizable(true))
            .column(Column::initial(120.0).resizable(true))
            .column(Column::initial(160.0).resizable(true))
            .column(Column::initial(160.0).resizable(true))
            .column(Column::initial(60.0).resizable(true))
            .header(18.0, |mut header| {
                header.col(|ui| { ui.strong("Datum"); });
                header.col(|ui| { ui.strong("Status"); });
                header.col(|ui| { ui.strong("Anlage"); });
                header.col(|ui| { ui.strong("SAP"); });
                header.col(|ui| { ui.strong("Prio"); });
                header.col(|ui| { ui.strong("Letzte Sch."); });
                header.col(|ui| { ui.strong("Bearbeiter"); });
                header.col(|ui| { ui.strong("Verantwortlich"); });
                header.col(|ui| { ui.strong("Störmeldung"); });
                header.col(|ui| { ui.strong("Maßnahme"); });
                header.col(|ui| { ui.strong("Bearb."); });
            })
            .body(|mut body| {
                for e in self.current_entries.clone() {
                    body.row(16.0, |mut row| {
                        row.col(|ui| { ui.label(e.date.to_string()); });
                        row.col(|ui| { ui.label(e.status.as_str()); });
                        row.col(|ui| { ui.label(&e.plant); });
                        row.col(|ui| {
                            if !e.sap_order.is_empty() && ui.small_button(&e.sap_order).clicked() {
                                ui.ctx().copy_text(e.sap_order.clone());
                                copied_sap = e.sap_order.clone();
                            }
                        });
                        row.col(|ui| { ui.label(e.priority.map(|p| p.to_string()).unwrap_or_default()); });
                        row.col(|ui| { ui.label(e.last_shift.map(|s| s.as_str().to_string()).unwrap_or_default()); });
                        row.col(|ui| { ui.label(&e.last_editor); });
                        row.col(|ui| { ui.label(&e.responsible); });
                        row.col(|ui| { ui.label(&e.fault); });
                        row.col(|ui| { ui.label(&e.action); });
                        row.col(|ui| {
                            if ui.small_button("Bearb.").clicked() {
                                clicked_id = Some(e.id.clone());
                            }
                        });
                        let resp = row.response();
                        if resp.clicked() {
                            detail_id = Some(e.id.clone());
                        }
                    });
                }
            });
        if !copied_sap.is_empty() { self.status_msg = format!("SAP {} kopiert", copied_sap); }
        if let Some(id) = detail_id { self.detail_id = Some(id); }
        if let Some(id) = clicked_id {
            if let Some(e) = self.current_entries.iter().find(|x| x.id == id).cloned() {
                self.load_edit(&e);
                self.show_edit_window = true;
            }
        }

    }

    fn ui_prio(&mut self, ui: &mut egui::Ui) {
        if ui.button("Aktualisieren").clicked() { self.reload(); }
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::initial(50.0).resizable(true))
            .column(Column::initial(90.0).resizable(true))
            .column(Column::initial(150.0).resizable(true))
            .column(Column::initial(110.0).resizable(true))
            .column(Column::initial(250.0).resizable(true))
            .header(18.0, |mut header| {
                header.col(|ui| { ui.strong("Prio"); });
                header.col(|ui| { ui.strong("Datum"); });
                header.col(|ui| { ui.strong("Anlage"); });
                header.col(|ui| { ui.strong("Status"); });
                header.col(|ui| { ui.strong("Störmeldung"); });
            })
            .body(|mut body| {
                for e in self.prio_entries.clone() {
                    body.row(16.0, |mut row| {
                        row.col(|ui| { ui.label(e.priority.map(|p| p.to_string()).unwrap_or_default()); });
                        row.col(|ui| { ui.label(e.date.to_string()); });
                        row.col(|ui| { ui.label(&e.plant); });
                        row.col(|ui| { ui.label(e.status.as_str()); });
                        row.col(|ui| { ui.label(&e.fault); });
                    });
                }
            });
    }

    fn ui_archiv(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Suche:"); ui.text_edit_singleline(&mut self.archive_query);
            ui.label("Anlage:");
            egui::ComboBox::from_id_salt("arch_plant").selected_text(if self.archive_filter_plant.is_empty() {"(alle)"} else {&self.archive_filter_plant}).show_ui(ui, |ui| {
                if ui.selectable_value(&mut self.archive_filter_plant, String::new(), "(alle)").clicked() {}
                for p in &self.plants.clone() { ui.selectable_value(&mut self.archive_filter_plant, p.clone(), p); }
            });
            if ui.button("Suchen").clicked() { self.reload(); }
        });
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::initial(90.0).resizable(true))
            .column(Column::initial(150.0).resizable(true))
            .column(Column::initial(110.0).resizable(true))
            .column(Column::initial(90.0).resizable(true))
            .column(Column::initial(250.0).resizable(true))
            .header(18.0, |mut header| {
                header.col(|ui| { ui.strong("Datum"); });
                header.col(|ui| { ui.strong("Anlage"); });
                header.col(|ui| { ui.strong("Status"); });
                header.col(|ui| { ui.strong("SAP"); });
                header.col(|ui| { ui.strong("Störmeldung"); });
            })
            .body(|mut body| {
                for e in self.archive_results.clone() {
                    body.row(16.0, |mut row| {
                        row.col(|ui| { ui.label(e.date.to_string()); });
                        row.col(|ui| { ui.label(&e.plant); });
                        row.col(|ui| { ui.label(e.status.as_str()); });
                        row.col(|ui| { ui.label(&e.sap_order); });
                        row.col(|ui| { ui.label(&e.fault); });
                    });
                }
            });
    }

    fn ui_shift_view(&mut self, ui: &mut egui::Ui, shift: &str) {
        ui.horizontal(|ui| {
            ui.label("Datum:"); ui.text_edit_singleline(&mut self.sicht_date);
            if ui.button("Laden").clicked() { self.reload(); }
        });
        ui.separator();
        let date: Option<NaiveDate> = self.sicht_date.parse().ok();
        if let Some(d) = date {
            let entries = self.store.as_ref().map(|st| st.list_current_shift(d, Shift::from_str(shift)).unwrap_or_default()).unwrap_or_default();
            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(Column::initial(90.0).resizable(true))
                .column(Column::initial(120.0).resizable(true))
                .column(Column::initial(80.0).resizable(true))
                .column(Column::initial(160.0).resizable(true))
                .column(Column::initial(160.0).resizable(true))
                .column(Column::initial(100.0).resizable(true))
                .header(16.0, |mut h| {
                    h.col(|ui| { ui.strong("Status"); });
                    h.col(|ui| { ui.strong("Anlage"); });
                    h.col(|ui| { ui.strong("Prio"); });
                    h.col(|ui| { ui.strong("Störmeldung"); });
                    h.col(|ui| { ui.strong("Maßnahme"); });
                    h.col(|ui| { ui.strong("Bearbeiter"); });
                })
                .body(|mut body| {
                    for e in entries {
                        body.row(16.0, |mut row| {
                            row.col(|ui| { ui.label(e.status.as_str()); });
                            row.col(|ui| { ui.label(&e.plant); });
                            row.col(|ui| { ui.label(e.priority.map(|p| p.to_string()).unwrap_or_default()); });
                            row.col(|ui| { ui.label(&e.fault); });
                            row.col(|ui| { ui.label(&e.action); });
                            row.col(|ui| { ui.label(&e.last_editor); });
                        });
                    }
                });
        }
    }

    fn ui_stamm(&mut self, ui: &mut egui::Ui) {
        ui.label("Anlagen");
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            for p in self.plants.clone() {
                if ui.selectable_label(self.selected_plant.as_deref() == Some(&p), &p).clicked() {
                    self.selected_plant = Some(p.clone());
                    self.rename_plant_name = p.clone();
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Neu:");
            ui.text_edit_singleline(&mut self.new_plant_name);
            if ui.button("Hinzufügen").clicked() && !self.new_plant_name.trim().is_empty() {
                if let Some(store) = self.store.as_mut() {
                    let _ = store.add_plant(self.new_plant_name.trim());
                    self.status_msg = "Anlage hinzugefügt".into();
                    self.new_plant_name.clear();
                    self.reload();
                }
            }
        });
        if let Some(sel) = self.selected_plant.clone() {
            ui.horizontal(|ui| {
                ui.label("Umbenennen:");
                ui.text_edit_singleline(&mut self.rename_plant_name);
                if ui.button("Speichern").clicked() && !self.rename_plant_name.trim().is_empty() {
                    if let Some(store) = self.store.as_mut() {
                        let _ = store.rename_plant(&sel, self.rename_plant_name.trim());
                        self.status_msg = "Anlage umbenannt".into();
                        self.selected_plant = None;
                        self.reload();
                    }
                }
                if ui.button("Löschen").clicked() {
                    if let Some(store) = self.store.as_mut() {
                        let _ = store.delete_plant(&sel);
                        self.status_msg = "Anlage gelöscht".into();
                        self.selected_plant = None;
                        self.reload();
                    }
                }
            });
        }
        ui.separator();
        ui.label("Verantwortliche");
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            for r in self.responsible.clone() {
                if ui.selectable_label(self.selected_resp.as_deref() == Some(&r), &r).clicked() {
                    self.selected_resp = Some(r.clone());
                    self.rename_resp_name = r.clone();
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Neu:");
            ui.text_edit_singleline(&mut self.new_resp_name);
            if ui.button("Hinzufügen").clicked() && !self.new_resp_name.trim().is_empty() {
                if let Some(store) = self.store.as_mut() {
                    let _ = store.add_responsible(self.new_resp_name.trim());
                    self.status_msg = "Verantwortlicher hinzugefügt".into();
                    self.new_resp_name.clear();
                    self.reload();
                }
            }
        });
        if let Some(sel) = self.selected_resp.clone() {
            ui.horizontal(|ui| {
                ui.label("Umbenennen:");
                ui.text_edit_singleline(&mut self.rename_resp_name);
                if ui.button("Speichern").clicked() && !self.rename_resp_name.trim().is_empty() {
                    if let Some(store) = self.store.as_mut() {
                        let _ = store.rename_responsible(&sel, self.rename_resp_name.trim());
                        self.status_msg = "Verantwortlicher umbenannt".into();
                        self.selected_resp = None;
                        self.reload();
                    }
                }
                if ui.button("Löschen").clicked() {
                    if let Some(store) = self.store.as_mut() {
                        let _ = store.delete_responsible(&sel);
                        self.status_msg = "Verantwortlicher gelöscht".into();
                        self.selected_resp = None;
                        self.reload();
                    }
                }
            });
        }
    }

    fn ui_uebergabe(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| { ui.label("Datum"); ui.text_edit_singleline(&mut self.ho_date); });
        ui.horizontal(|ui| { ui.label("Von Schicht"); ui.text_edit_singleline(&mut self.ho_from); });
        ui.horizontal(|ui| { ui.label("Nach Schicht"); ui.text_edit_singleline(&mut self.ho_to); });
        ui.horizontal(|ui| { ui.label("Benutzer"); ui.text_edit_singleline(&mut self.ho_user); });
        if ui.button("Schichtübergabe durchführen").clicked() {
            self.do_handover();
        }
        ui.label("Erledigte Aufträge werden archiviert, offene übernommen.");
    }
}
