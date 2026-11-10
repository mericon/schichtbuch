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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Offen,
    InBearbeitung,
    StehtInBearbeitung,
    Erledigt,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Offen => "offen",
            Status::InBearbeitung => "In Bearbeitung",
            Status::StehtInBearbeitung => "steht in Bearbeitung",
            Status::Erledigt => "erledigt",
        }
    }
    pub fn from_str(s: &str) -> Self {
        let l = s.to_lowercase();
        if l.contains("erledigt") { Status::Erledigt }
        else if l.contains("steht") { Status::StehtInBearbeitung }
        else if l.contains("bearbeitung") { Status::InBearbeitung }
        else { Status::Offen }
    }
    pub fn is_done(&self) -> bool { matches!(self, Status::Erledigt) }
    pub fn is_open(&self) -> bool { !self.is_done() }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub date: NaiveDate,
    pub shift: Shift,
    pub last_shift: Option<Shift>,
    pub last_editor: String,
    pub status: Status,
    pub priority: Option<i32>,
    pub plant: String,
    pub sap_order: String,
    pub fault: String,
    pub responsible: String,
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
            shift,
            last_shift: None,
            last_editor: String::new(),
            status: Status::Offen,
            priority: None,
            plant: String::new(),
            sap_order: String::new(),
            fault: String::new(),
            responsible: String::new(),
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
    pub performed_at: NaiveDateTime,
    pub archived_count: i64,
    pub carried_count: i64,
}

