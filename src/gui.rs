use crate::domain::{Entry, Shift};
use crate::storage::Store;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column as IcedColumn};
use iced::{Element, Length, Task};
use std::path::PathBuf;

pub fn run(db: PathBuf) {
    let boot = move || Schichtbuch::new(db.clone());
    let _ = iced::application(boot, Schichtbuch::update, Schichtbuch::view).run();
}

pub struct Schichtbuch {
    db_path: PathBuf,
    store: Option<Store>,
    entries: Vec<Entry>,
    status: String,
    new_plant: String,
    new_fault: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded,
    Refresh,
    NewPlantChanged(String),
    NewFaultChanged(String),
    AddEntry,
}

impl Schichtbuch {
    fn new(db: PathBuf) -> Self {
        let store = Store::open(&db).ok();
        let mut app = Self {
            db_path: db,
            store,
            entries: vec![],
            status: String::new(),
            new_plant: String::new(),
            new_fault: String::new(),
        };
        app.reload();
        app
    }

    fn reload(&mut self) {
        if let Some(store) = self.store.as_ref() {
            let all = store.list_entries(false).unwrap_or_default();
            self.entries = all.into_iter().filter(|e| e.status != "geplant").collect();
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Loaded | Message::Refresh => {
                self.reload();
                iced::Task::none()
            }
            Message::NewPlantChanged(v) => {
                self.new_plant = v;
                iced::Task::none()
            }
            Message::NewFaultChanged(v) => {
                self.new_fault = v;
                iced::Task::none()
            }
            Message::AddEntry => {
                if let Some(store) = self.store.as_mut() {
                    let entry = Entry::new(chrono::Local::now().date_naive(), Shift::Frueh);
                    let mut e = entry;
                    e.plant = self.new_plant.clone();
                    e.fault = self.new_fault.clone();
                    e.status = "offen".to_string();
                    let _ = store.insert_entry(&e);
                    self.new_plant.clear();
                    self.new_fault.clear();
                    self.reload();
                }
                iced::Task::none()
            }
        }
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        let title = text("Schichtbuch").size(24);
        let mut col = column![title].spacing(10).padding(10);

        // new entry form
        col = col.push(row![
            text_input("Anlage", &self.new_plant).on_input(Message::NewPlantChanged),
            text_input("Störmeldung", &self.new_fault).on_input(Message::NewFaultChanged),
            button("Hinzufügen").on_press(Message::AddEntry),
        ].spacing(8));

        col = col.push(text(format!("{} Einträge", self.entries.len())));

        // entries list
        let mut list = column![].spacing(4);
        for e in &self.entries {
            list = list.push(row![
                text(e.date.to_string()).width(Length::Fixed(100.0)),
                text(&e.plant).width(Length::Fixed(150.0)),
                text(&e.status).width(Length::Fixed(100.0)),
                text(&e.fault).width(Length::Fixed(200.0)),
            ].spacing(8));
        }
        col = col.push(scrollable(list).height(Length::Fill));

        container(col).into()
    }
}
