use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shift {
    Frueh,
    Spaet,
    Nacht,
    Tag,
}

impl Shift {
    pub fn as_str(&self) -> &'static str {
        match self {
            Shift::Frueh => "Früh",
            Shift::Spaet => "Spät",
            Shift::Nacht => "Nacht",
            Shift::Tag => "Tag",
        }
    }
    pub fn from_str(s: &str) -> Self {
        let l = s.to_lowercase();
        if l.contains("spät") || l.contains("spat") { Shift::Spaet }
        else if l.contains("nacht") { Shift::Nacht }
        else if l.contains("tag") { Shift::Tag }
        else { Shift::Frueh }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub date: NaiveDate,
    pub time: String,
    pub shift: Shift,
    pub last_shift: Option<Shift>,
    pub last_editor: String,
    pub version: i64,
    pub status: String,
    pub priority: Option<i32>,
    pub plant: String,
    pub sap_order: String,
    pub fault: String,
    pub responsible: String,
    pub gewerk: String,
    pub wartung: bool,
    pub kommentar: String,
    pub handed_over_to: String,
    pub downtime: String,
    pub action: String,
    pub images: String,
    pub transferred: bool,
    pub archived: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub created_by: String,
}

impl Entry {
    pub fn new(date: NaiveDate, shift: Shift) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4().to_string(),
            date,
            time: chrono::Local::now().format("%H:%M").to_string(),
            shift,
            last_shift: None,
            last_editor: String::new(),
            version: 0,
            status: "offen".to_string(),
            priority: None,
            plant: String::new(),
            sap_order: String::new(),
            fault: String::new(),
            responsible: String::new(),
            gewerk: String::new(),
            wartung: false,
            kommentar: String::new(),
            handed_over_to: String::new(),
            downtime: String::new(),
            action: String::new(),
            images: String::new(),
            transferred: false,
            archived: false,
            created_at: now,
            updated_at: now,
            created_by: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handover {
    pub id: String,
    pub from_shift: Shift,
    pub to_shift: Shift,
    pub date: NaiveDate,
    pub performed_by: String,
    pub taken_over_by: String,
    pub performed_at: NaiveDateTime,
    pub archived_count: i64,
    pub carried_count: i64,
}


pub fn status_is_done(name: &str) -> bool {
    name.to_lowercase().contains("erledigt")
}

pub fn status_rgb(name: &str) -> (u8, u8, u8) {
    let l = name.to_lowercase();
    if l.contains("erledigt") { (34, 139, 34) }
    else if l.contains("steht") && l.contains("offen") { (220, 20, 60) }
    else if l.contains("steht") { (220, 20, 60) }
    else if l.contains("geplant") { (186, 85, 211) }
    else if l.contains("bearbeitung") { (255, 215, 0) }
    else if l.contains("offen") { (255, 165, 0) }
    else { (128, 128, 128) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub timestamp: NaiveDateTime,
    pub event_type: String,
    pub user: String,
    pub details: String,
    pub related_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Responsible {
    pub name: String,
    pub amain: String,
    pub telefon: String,
    pub schicht_vk: String,
    pub gewerk: String,
}

#[derive(Debug, Clone, Default)]
pub struct EntryForm {
    pub date: String,
    pub time: String,
    pub shift: String,
    pub plant: String,
    pub status: String,
    pub sap: String,
    pub fault: String,
    pub responsible: String,
    pub gewerk: String,
    pub priority: String,
    pub action: String,
    pub downtime: String,
    pub handed: String,
    pub kommentar: String,
    pub images: String,
}

impl EntryForm {
    pub fn from_entry(e: &Entry) -> Self {
        Self {
            date: e.date.to_string(),
            time: e.time.clone(),
            shift: e.shift.as_str().to_string(),
            plant: e.plant.clone(),
            status: e.status.clone(),
            sap: e.sap_order.clone(),
            fault: e.fault.clone(),
            responsible: e.responsible.clone(),
            gewerk: e.gewerk.clone(),
            priority: e.priority.map(|p| p.to_string()).unwrap_or_default(),
            action: e.action.clone(),
            downtime: e.downtime.clone(),
            handed: e.handed_over_to.clone(),
            kommentar: e.kommentar.clone(),
            images: e.images.clone(),
        }
    }
    pub fn apply_to(&self, e: &mut Entry) {
        if let Ok(d) = self.date.parse() { e.date = d; }
        e.time = self.time.clone();
        e.shift = Shift::from_str(&self.shift);
        e.plant = self.plant.clone();
        e.status = self.status.clone();
        e.sap_order = self.sap.clone();
        e.fault = self.fault.clone();
        e.responsible = self.responsible.clone();
        e.gewerk = self.gewerk.clone();
        e.priority = self.priority.trim().parse().ok();
        e.action = self.action.clone();
        e.downtime = self.downtime.clone();
        e.handed_over_to = self.handed.clone();
        e.kommentar = self.kommentar.clone();
        e.images = self.images.clone();
    }
}
