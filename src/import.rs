use crate::domain::{Entry, Shift};
use crate::storage::Store;
use calamine::{open_workbook_auto, Data, Reader};
use chrono::NaiveDate;
use std::path::Path;

fn to_date(v: &Data) -> Option<NaiveDate> {
    match v {
        Data::DateTime(d) => d.as_datetime().map(|dt| dt.date()),
        Data::DateTimeIso(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
            .or_else(|| NaiveDate::parse_from_str(s, "%d.%m.%Y").ok()),
        Data::Float(f) => excel_serial_to_date(*f),
        Data::Int(i) => excel_serial_to_date(*i as f64),
        Data::String(s) => {
            let s = s.trim();
            NaiveDate::parse_from_str(s, "%d.%m.%Y")
                .or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
                .or_else(|_| NaiveDate::parse_from_str(s, "%d/%m/%Y"))
                .ok()
        }
        _ => None,
    }
}

fn excel_serial_to_date(serial: f64) -> Option<NaiveDate> {
    // Excel serial date: days since 1899-12-30
    if serial < 1.0 { return None; }
    let days = serial as i64;
    let base = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    base.checked_add_days(chrono::Days::new(days as u64))
}

fn to_str(v: &Data) -> String {
    match v {
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if *f == f.trunc() { format!("{}", *f as i64) } else { format!("{}", f) }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => d.as_datetime().map(|dt| dt.date().to_string()).unwrap_or_default(),
        Data::DateTimeIso(s) => s.clone(),
        Data::Empty => String::new(),
        _ => String::new(),
    }
}

fn to_i64(v: &Data) -> Option<i64> {
    match v {
        Data::Int(i) => Some(*i),
        Data::Float(f) => Some(*f as i64),
        Data::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn cell(row: &[Data], idx: usize) -> Data {
    row.get(idx).cloned().unwrap_or(Data::Empty)
}

pub fn import_workbook(store: &Store, path: &Path) -> Result<usize, String> {
    let mut workbook = open_workbook_auto(path).map_err(|e| e.to_string())?;
    let mut total = 0usize;

    // Master data
    if let Ok(range) = workbook.worksheet_range("Stamm_Anlagen") {
        for row in range.rows().skip(1) {
            let name = to_str(&cell(row, 0)).trim().to_string();
            if !name.is_empty() {
                let _ = store.add_plant(&name);
            }
        }
    }
    if let Ok(range) = workbook.worksheet_range("Stamm_Verantwortlich") {
        for row in range.rows().skip(1) {
            let name = to_str(&cell(row, 0)).trim().to_string();
            if !name.is_empty() {
                let _ = store.add_responsible(&name);
            }
        }
    }

    // Historie
    if let Ok(range) = workbook.worksheet_range("Historie") {
        for row in range.rows() {
            // skip header
            let c0 = cell(row, 0);
            if matches!(c0, Data::String(ref s) if s.contains("Start Datum") || s.contains("Datum")) { continue; }
            let date = match to_date(&c0) { Some(d) => d, None => continue };
            let status_str = to_str(&cell(row, 1));
            if status_str.trim().is_empty() && to_str(&cell(row, 2)).trim().is_empty() { continue; }
            let mut e = Entry::new(date, Shift::Frueh);
            e.status = status_str.clone();
            e.plant = to_str(&cell(row, 2)).trim().to_string();
            e.sap_order = to_str(&cell(row, 3)).trim().to_string();
            e.fault = to_str(&cell(row, 4)).trim().to_string();
            e.responsible = to_str(&cell(row, 5)).trim().to_string();
            e.handed_over_to = to_str(&cell(row, 6)).trim().to_string();
            e.downtime = to_str(&cell(row, 7)).trim().to_string();
            e.action = to_str(&cell(row, 8)).trim().to_string();
            e.images = to_str(&cell(row, 9)).trim().to_string();
            e.archived = crate::domain::status_is_done(&e.status);
            e.created_by = "import".into();
            // shift unknown in Historie; default Früh. Try infer from? no.
            let _ = store.insert_entry(&e);
            total += 1;
        }
    }

    // Prio offen
    if let Ok(range) = workbook.worksheet_range("Prio offen") {
        for row in range.rows() {
            // header detection
            let c0 = cell(row, 0);
            if matches!(c0, Data::String(ref s) if s == "Prio" || s == "Prio offen") { continue; }
            // rows may have prio in col0
            // find date: try col1 first, then col0
            let (prio, date, status_idx, plant_idx) = if let Some(d) = to_date(&cell(row, 1)) {
                (to_i64(&c0), Some(d), 2usize, 3usize)
            } else if let Some(d) = to_date(&c0) {
                (None, Some(d), 1usize, 2usize)
            } else {
                (None, None, 0usize, 0usize)
            };
            let date = match date { Some(d) => d, None => continue };
            let status_str = to_str(&cell(row, status_idx));
            if status_str.trim().is_empty() { 
                // maybe row like 'Fremdfirmen'
                continue;
            }
            let mut e = Entry::new(date, Shift::Frueh);
            e.priority = prio.map(|p| p as i32);
            e.status = status_str.clone();
            e.plant = to_str(&cell(row, plant_idx)).trim().to_string();
            e.sap_order = to_str(&cell(row, plant_idx + 1)).trim().to_string();
            e.fault = to_str(&cell(row, plant_idx + 2)).trim().to_string();
            e.responsible = to_str(&cell(row, plant_idx + 3)).trim().to_string();
            e.handed_over_to = to_str(&cell(row, plant_idx + 4)).trim().to_string();
            e.downtime = to_str(&cell(row, plant_idx + 5)).trim().to_string();
            e.action = to_str(&cell(row, plant_idx + 6)).trim().to_string();
            // images col
            e.archived = false;
            e.created_by = "import".into();
            let _ = store.insert_entry(&e);
            total += 1;
        }
    }

    Ok(total)
}

