#![cfg_attr(windows, windows_subsystem = "windows")]
mod domain;
mod gui;
mod import;
mod storage;

use crate::domain::{Entry, Shift, Status};
use crate::storage::Store;
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "schichtbuch", version, about = "Schichtbuch - shift log")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init { db: PathBuf },
    Import { xlsx: PathBuf, #[arg(long)] db: PathBuf },
    Stats { #[arg(long)] db: PathBuf },
    List { #[arg(long)] db: PathBuf, #[arg(long)] archived: bool, #[arg(long)] shift: Option<String>, #[arg(long)] date: Option<String> },
    Prio { #[arg(long)] db: PathBuf },
    Search { #[arg(long)] db: PathBuf, query: String },
    Add {
        #[arg(long)] db: PathBuf,
        #[arg(long)] date: String,
        #[arg(long)] shift: String,
        #[arg(long)] plant: String,
        #[arg(long)] status: Option<String>,
        #[arg(long)] sap: Option<String>,
        #[arg(long)] fault: Option<String>,
        #[arg(long)] responsible: Option<String>,
        #[arg(long)] priority: Option<i32>,
        #[arg(long)] action: Option<String>,
    },
    UpdateStatus { #[arg(long)] db: PathBuf, id: String, status: String },
    SetPriority { #[arg(long)] db: PathBuf, id: String, priority: i32 },
    Archive { #[arg(long)] db: PathBuf, #[arg(long)] date: Option<String>, #[arg(long)] shift: Option<String> },
    Handover {
        #[arg(long)] db: PathBuf,
        #[arg(long)] from: String,
        #[arg(long)] to: String,
        #[arg(long)] date: String,
        #[arg(long)] user: String,
    },
    Handovers { #[arg(long)] db: PathBuf },
    Gui { #[arg(long)] db: Option<PathBuf> },
    Plants { #[arg(long)] db: PathBuf },
    Responsible { #[arg(long)] db: PathBuf },
}

fn open(db: &PathBuf) -> Store {
    Store::open(db).expect("cannot open db")
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Commands::Init { db } => {
            let s = Store::open(&db).expect("init failed");
            println!("initialized {}", db.display());
            let _ = s;
        }
        Commands::Import { xlsx, db } => {
            let store = open(&db);
            match import::import_workbook(&store, &xlsx) {
                Ok(n) => println!("imported {} entries", n),
                Err(e) => eprintln!("import error: {}", e),
            }
        }
        Commands::Stats { db } => {
            let store = open(&db);
            println!("open entries: {}", store.count_entries(false).unwrap_or(0));
            println!("archived entries: {}", store.count_entries(true).unwrap_or(0));
            println!("plants: {}", store.list_plants().unwrap_or_default().len());
            println!("responsible: {}", store.list_responsible().unwrap_or_default().len());
        }
        Commands::List { db, archived, shift, date } => {
            let store = open(&db);
            let entries = if let (Some(d), Some(sh)) = (date.as_ref(), shift.as_ref()) {
                let dd: NaiveDate = d.parse().expect("bad date");
                store.list_current_shift(dd, Shift::from_str(sh)).unwrap_or_default()
            } else {
                store.list_entries(archived).unwrap_or_default()
            };
            for e in entries {
                println!("{} | {} | {} | {} | prio={:?} | {} | {}", e.id, e.date, e.shift.as_str(), e.status.as_str(), e.priority, e.plant, e.sap_order);
            }
        }
        Commands::Prio { db } => {
            let store = open(&db);
            for e in store.list_open_priorities().unwrap_or_default() {
                println!("{:?} | {} | {} | {} | {}", e.priority, e.date, e.plant, e.status.as_str(), e.fault);
            }
        }
        Commands::Search { db, query } => {
            let store = open(&db);
            for e in store.search_archive(&query).unwrap_or_default() {
                println!("{} | {} | {} | {} | {}", e.date, e.plant, e.status.as_str(), e.sap_order, e.fault);
            }
        }
        Commands::Add { db, date, shift, plant, status, sap, fault, responsible, priority, action } => {
            let store = open(&db);
            let dd: NaiveDate = date.parse().expect("bad date");
            let mut e = Entry::new(dd, Shift::from_str(&shift));
            e.plant = plant;
            if let Some(s) = status { e.status = Status::from_str(&s); }
            e.sap_order = sap.unwrap_or_default();
            e.fault = fault.unwrap_or_default();
            e.responsible = responsible.unwrap_or_default();
            e.priority = priority;
            e.action = action.unwrap_or_default();
            store.insert_entry(&e).expect("insert failed");
            println!("added {}", e.id);
        }
        Commands::UpdateStatus { db, id, status } => {
            let store = open(&db);
            if let Some(mut e) = store.get_entry(&id).expect("get failed") {
                e.status = Status::from_str(&status);
                e.updated_at = chrono::Local::now().naive_local();
                store.update_entry(&e).expect("update failed");
                println!("updated {}", id);
            } else {
                eprintln!("not found");
            }
        }
        Commands::SetPriority { db, id, priority } => {
            let store = open(&db);
            if let Some(mut e) = store.get_entry(&id).expect("get failed") {
                e.priority = Some(priority);
                e.updated_at = chrono::Local::now().naive_local();
                store.update_entry(&e).expect("update failed");
                println!("priority set {}", id);
            } else {
                eprintln!("not found");
            }
        }
        Commands::Archive { db, date, shift } => {
            let store = open(&db);
            let d = date.as_ref().and_then(|s| s.parse().ok());
            let sh = shift.as_ref().map(|s| Shift::from_str(s));
            let n = store.archive_completed(d, sh).expect("archive failed");
            println!("archived {} entries", n);
        }
        Commands::Handover { db, from, to, date, user } => {
            let store = open(&db);
            let dd: NaiveDate = date.parse().expect("bad date");
            let fromsh = Shift::from_str(&from);
            let tosh = Shift::from_str(&to);
            // archive completed for this shift/date
            let archived = store.archive_completed(Some(dd), Some(fromsh)).unwrap_or(0);
            // carried = open entries remaining
            let open = store.list_open_priorities().unwrap_or_default();
            let carried = open.len() as i64;
            let h = crate::domain::Handover {
                id: uuid::Uuid::new_v4().to_string(),
                from_shift: fromsh,
                to_shift: tosh,
                date: dd,
                performed_by: user.clone(),
                performed_at: chrono::Local::now().naive_local(),
                archived_count: archived,
                carried_count: carried,
            };
            store.record_handover(&h).expect("handover failed");
            println!("handover {} -> {}: archived={}, carried={}", from, to, archived, carried);
        }
        Commands::Handovers { db } => {
            let store = open(&db);
            for h in store.list_handovers().unwrap_or_default() {
                println!("{} | {} -> {} | {} | by {} | archived={} carried={}", h.performed_at, h.from_shift.as_str(), h.to_shift.as_str(), h.date, h.performed_by, h.archived_count, h.carried_count);
            }
        }
        Commands::Plants { db } => {
            let store = open(&db);
            for p in store.list_plants().unwrap_or_default() { println!("{}", p); }
        }
        Commands::Responsible { db } => {
            let store = open(&db);
            for r in store.list_responsible().unwrap_or_default() { println!("{}", r); }
        }
        Commands::Gui { db } => {
            let path = db.unwrap_or_else(|| PathBuf::from("schichtbuch.db"));
            if !path.exists() { let _ = Store::open(&path); }
            gui::run(path);
        }
    }
}
