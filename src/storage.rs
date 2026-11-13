use crate::domain::{Entry, Handover, Shift};
use chrono::NaiveDate;
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct Store {
    pub conn: Connection,
}

fn is_network_path(p: &std::path::Path) -> bool {
    let s = p.to_string_lossy();
    s.starts_with(r"\\") || s.starts_with("//")
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        // multi-user friendly settings
        conn.busy_timeout(std::time::Duration::from_secs(30))?;

        let want_wal = !is_network_path(path);
        let target = if want_wal { "WAL" } else { "DELETE" };

        let actual: String = conn.query_row(
            &format!("PRAGMA journal_mode = {}", target), [], |r| r.get(0))?;

        if !actual.eq_ignore_ascii_case(target) {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
                Some(format!("journal_mode ist '{}', erwartet '{}'", actual, target)),
            ));
        }

        conn.pragma_update(None, "synchronous", if want_wal { "NORMAL" } else { "FULL" })?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }


    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS entries (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL,
                time TEXT NOT NULL DEFAULT '',
                shift TEXT NOT NULL,
                status TEXT NOT NULL,
                priority INTEGER,
                plant TEXT NOT NULL DEFAULT '',
                sap_order TEXT NOT NULL DEFAULT '',
                fault TEXT NOT NULL DEFAULT '',
                responsible TEXT NOT NULL DEFAULT '',
                gewerk TEXT NOT NULL DEFAULT '',
                wartung INTEGER NOT NULL DEFAULT 0,
                kommentar TEXT NOT NULL DEFAULT '',
                handed_over_to TEXT NOT NULL DEFAULT '',
                downtime TEXT NOT NULL DEFAULT '',
                action TEXT NOT NULL DEFAULT '',
                images TEXT NOT NULL DEFAULT '',
                transferred INTEGER NOT NULL DEFAULT 0,
                archived INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                created_by TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_entries_date_shift ON entries(date, shift);
            CREATE INDEX IF NOT EXISTS idx_entries_archived ON entries(archived);
            CREATE INDEX IF NOT EXISTS idx_entries_status ON entries(status);
            CREATE INDEX IF NOT EXISTS idx_entries_priority ON entries(priority);

            CREATE TABLE IF NOT EXISTS handovers (
                id TEXT PRIMARY KEY,
                from_shift TEXT NOT NULL,
                to_shift TEXT NOT NULL,
                date TEXT NOT NULL,
                performed_by TEXT NOT NULL DEFAULT '',
                performed_at TEXT NOT NULL,
                archived_count INTEGER NOT NULL DEFAULT 0,
                carried_count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS master_plants (name TEXT PRIMARY KEY);
            CREATE TABLE IF NOT EXISTS master_responsible (name TEXT PRIMARY KEY, amain TEXT NOT NULL DEFAULT '', telefon TEXT NOT NULL DEFAULT '', schicht_vk TEXT NOT NULL DEFAULT '', gewerk TEXT NOT NULL DEFAULT '');
            CREATE TABLE IF NOT EXISTS master_statuses (name TEXT PRIMARY KEY, weight INTEGER NOT NULL DEFAULT 0);
            CREATE TABLE IF NOT EXISTS master_shifts (name TEXT PRIMARY KEY);
            CREATE TABLE IF NOT EXISTS master_gewerke (name TEXT PRIMARY KEY);

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                user TEXT NOT NULL DEFAULT '',
                details TEXT NOT NULL DEFAULT '',
                related_id TEXT NOT NULL DEFAULT ''
            );
            "#,
        )?;
        // migration: add last_shift column if missing
        let _ = self.conn.execute("ALTER TABLE entries ADD COLUMN last_shift TEXT", []);
        let _ = self.conn.execute("ALTER TABLE entries ADD COLUMN last_editor TEXT", []);
        let _ = self.conn.execute("ALTER TABLE entries ADD COLUMN time TEXT DEFAULT ''", []);
        let _ = self.conn.execute("ALTER TABLE entries ADD COLUMN gewerk TEXT NOT NULL DEFAULT ''", []);
        let _ = self.conn.execute("ALTER TABLE entries ADD COLUMN wartung INTEGER NOT NULL DEFAULT 0", []);
        let _ = self.conn.execute("ALTER TABLE entries ADD COLUMN kommentar TEXT NOT NULL DEFAULT ''", []);
        let _ = self.conn.execute("ALTER TABLE master_statuses ADD COLUMN weight INTEGER NOT NULL DEFAULT 0", []);
        let _ = self.conn.execute("ALTER TABLE master_responsible ADD COLUMN amain TEXT NOT NULL DEFAULT ''", []);
        let _ = self.conn.execute("ALTER TABLE master_responsible ADD COLUMN telefon TEXT NOT NULL DEFAULT ''", []);
        let _ = self.conn.execute("ALTER TABLE master_responsible ADD COLUMN schicht_vk TEXT NOT NULL DEFAULT ''", []);
        let _ = self.conn.execute("ALTER TABLE master_responsible ADD COLUMN gewerk TEXT NOT NULL DEFAULT ''", []);
        let _ = self.conn.execute("ALTER TABLE handovers ADD COLUMN taken_over_by TEXT NOT NULL DEFAULT ''", []);
        // seed default statuses if empty
        let cnt: i64 = self.conn.query_row("SELECT COUNT(*) FROM master_statuses", [], |r| r.get(0)).unwrap_or(0);
        if cnt == 0 {
            let defaults = [("offen",1),("steht offen",2),("geplant",3),("In Bearbeitung",4),("steht in Bearbeitung",5),("erledigt",6)];
            for (n,w) in defaults {
                let _ = self.conn.execute("INSERT OR IGNORE INTO master_statuses (name, weight) VALUES (?1, ?2)", rusqlite::params![n, w]);
            }
        }
        Ok(())
    }

    pub fn insert_entry(&self, e: &Entry) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO entries (id,date,time,shift,last_shift,last_editor,status,priority,plant,sap_order,fault,responsible,gewerk,wartung,kommentar,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)"#,
            params![
                e.id,
                e.date.to_string(),
                e.time.clone(),
                e.shift.as_str(),
                e.last_shift.map(|s| s.as_str().to_string()),
                e.last_editor.clone(),
                e.status.as_str(),
                e.priority,
                e.plant,
                e.sap_order,
                e.fault,
                e.responsible,
                e.gewerk,
                e.wartung as i32,
                e.kommentar.clone(),
                e.handed_over_to,
                e.downtime,
                e.action,
                e.images,
                e.transferred as i32,
                e.archived as i32,
                e.created_at.to_string(),
                e.updated_at.to_string(),
                e.created_by,
            ],
        )?;
        Ok(())
    }

    pub fn update_entry(&self, e: &Entry) -> Result<()> {
        self.conn.execute(
            r#"UPDATE entries SET date=?2,time=?3,shift=?4,last_shift=?5,last_editor=?6,status=?7,priority=?8,plant=?9,sap_order=?10,fault=?11,responsible=?12,gewerk=?13,wartung=?14,kommentar=?15,handed_over_to=?16,downtime=?17,action=?18,images=?19,transferred=?20,archived=?21,updated_at=?22,created_by=?23
               WHERE id=?1"#,
            params![
                e.id,
                e.date.to_string(),
                e.time.clone(),
                e.shift.as_str(),
                e.last_shift.map(|s| s.as_str().to_string()),
                e.last_editor.clone(),
                e.status.as_str(),
                e.priority,
                e.plant,
                e.sap_order,
                e.fault,
                e.responsible,
                e.gewerk,
                e.wartung as i32,
                e.kommentar.clone(),
                e.handed_over_to,
                e.downtime,
                e.action,
                e.images,
                e.transferred as i32,
                e.archived as i32,
                e.updated_at.to_string(),
                e.created_by,
            ],
        )?;
        Ok(())
    }


    pub fn get_entry(&self, id: &str) -> Result<Option<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,date,time,shift,last_shift,last_editor,status,priority,plant,sap_order,fault,responsible,gewerk,wartung,kommentar,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by FROM entries WHERE id=?1"
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(r) = rows.next()? {
            Ok(Some(row_to_entry(r)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_entries(&self, archived: bool) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,date,time,shift,last_shift,last_editor,status,priority,plant,sap_order,fault,responsible,gewerk,wartung,kommentar,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by FROM entries WHERE archived=?1 ORDER BY date DESC, priority ASC NULLS LAST, created_at DESC"
        )?;
        let rows = stmt.query_map(params![archived as i32], |r| row_to_entry(r))?;
        let mut out = Vec::new();
        for e in rows { out.push(e?); }
        Ok(out)
    }

    pub fn list_current_shift(&self, date: NaiveDate, shift: Shift) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,date,time,shift,last_shift,last_editor,status,priority,plant,sap_order,fault,responsible,gewerk,wartung,kommentar,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by FROM entries WHERE archived=0 AND date=?1 AND shift=?2 ORDER BY priority ASC NULLS LAST, created_at ASC"
        )?;
        let rows = stmt.query_map(params![date.to_string(), shift.as_str()], |r| row_to_entry(r))?;
        let mut out = Vec::new();
        for e in rows { out.push(e?); }
        Ok(out)
    }

    pub fn list_open_priorities(&self) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,date,time,shift,last_shift,last_editor,status,priority,plant,sap_order,fault,responsible,gewerk,wartung,kommentar,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by FROM entries WHERE archived=0 AND status<>'erledigt' ORDER BY priority ASC NULLS LAST, date ASC, created_at ASC"
        )?;
        let rows = stmt.query_map([], |r| row_to_entry(r))?;
        let mut out = Vec::new();
        for e in rows { out.push(e?); }
        Ok(out)
    }

    /// Archive all completed entries for a given date/shift (or all completed if None)
    pub fn archive_completed(&self, date: Option<NaiveDate>, shift: Option<Shift>) -> Result<i64> {
        let mut sql = "UPDATE entries SET archived=1, updated_at=?1 WHERE archived=0 AND status='erledigt'".to_string();
        // build conditions
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        args.push(Box::new(chrono::Local::now().naive_local().to_string()));
        if let Some(d) = date {
            sql.push_str(" AND date=?2");
            args.push(Box::new(d.to_string()));
        }
        if let Some(sh) = shift {
            // shift condition index depends
            if date.is_some() {
                sql.push_str(" AND shift=?3");
            } else {
                sql.push_str(" AND shift=?2");
            }
            args.push(Box::new(sh.as_str().to_string()));
        }
        let n = self.conn.execute(&sql, rusqlite::params_from_iter(args.iter().map(|b| b.as_ref())))?;
        Ok(n as i64)
    }

    /// Carry over open (not done) entries from one shift to another (Übernahme).
    /// The original entries in `from` are frozen (kept, read-only) and copies are
    /// created in `to` (active). Frozen originals remain until overwritten by a
    /// later takeover.
    pub fn carry_over_open(&self, from: Shift, to: Shift, date: Option<NaiveDate>) -> Result<i64> {
        let mut sql = "UPDATE entries SET shift=?1, last_shift=?2, updated_at=?3 WHERE archived=0 AND status<>'erledigt' AND shift=?4".to_string();
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        args.push(Box::new(to.as_str().to_string()));
        args.push(Box::new(from.as_str().to_string()));
        args.push(Box::new(chrono::Local::now().naive_local().to_string()));
        args.push(Box::new(from.as_str().to_string()));
        if let Some(d) = date {
            sql.push_str(" AND date=?5");
            args.push(Box::new(d.to_string()));
        }
        let n = self.conn.execute(&sql, rusqlite::params_from_iter(args.iter().map(|b| b.as_ref())))?;
        Ok(n as i64)
    }

    pub fn search_archive(&self, query: &str) -> Result<Vec<Entry>> {
        let like = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id,date,time,shift,last_shift,last_editor,status,priority,plant,sap_order,fault,responsible,gewerk,wartung,kommentar,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by FROM entries WHERE archived=1 AND (plant LIKE ?1 OR fault LIKE ?1 OR action LIKE ?1 OR sap_order LIKE ?1 OR responsible LIKE ?1) ORDER BY date DESC, created_at DESC"
        )?;
        let rows = stmt.query_map(params![like], |r| row_to_entry(r))?;
        let mut out = Vec::new();
        for e in rows { out.push(e?); }
        Ok(out)
    }

    pub fn record_handover(&self, h: &Handover) -> Result<()> {
        self.conn.execute(
            "INSERT INTO handovers (id,from_shift,to_shift,date,performed_by,taken_over_by,performed_at,archived_count,carried_count) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![h.id, h.from_shift.as_str(), h.to_shift.as_str(), h.date.to_string(), h.performed_by, h.taken_over_by, h.performed_at.to_string(), h.archived_count, h.carried_count],
        )?;
        Ok(())
    }

    pub fn list_handovers(&self) -> Result<Vec<Handover>> {
        let mut stmt = self.conn.prepare("SELECT id,from_shift,to_shift,date,performed_by,taken_over_by,performed_at,archived_count,carried_count FROM handovers ORDER BY performed_at DESC")?;
        let rows = stmt.query_map([], |r| {
            Ok(Handover {
                id: r.get(0)?,
                from_shift: Shift::from_str(&r.get::<_,String>(1)?),
                to_shift: Shift::from_str(&r.get::<_,String>(2)?),
                date: r.get::<_,String>(3)?.parse().unwrap_or_else(|_| chrono::Local::now().date_naive()),
                performed_by: r.get(4)?,
                taken_over_by: r.get(5)?,
                performed_at: r.get::<_,String>(6)?.parse().unwrap_or_else(|_| chrono::Local::now().naive_local()),
                archived_count: r.get(7)?,
                carried_count: r.get(8)?,
            })
        })?;
        let mut out = Vec::new();
        for h in rows { out.push(h?); }
        Ok(out)
    }

    // master data
    pub fn add_plant(&self, name: &str) -> Result<()> {
        self.conn.execute("INSERT OR IGNORE INTO master_plants (name) VALUES (?1)", params![name])?;
        Ok(())
    }
    pub fn list_plants(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT name FROM master_plants ORDER BY name")?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        let mut out = Vec::new();
        for r in rows { out.push(r?); }
        Ok(out)
    }
    pub fn add_responsible(&self, name: &str) -> Result<()> {
        self.conn.execute("INSERT OR IGNORE INTO master_responsible (name) VALUES (?1)", params![name])?;
        Ok(())
    }
    pub fn list_responsible(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT name FROM master_responsible ORDER BY name")?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        let mut out = Vec::new();
        for r in rows { out.push(r?); }
        Ok(out)
    }

    pub fn list_responsible_full(&self) -> Result<Vec<crate::domain::Responsible>> {
        let mut stmt = self.conn.prepare("SELECT name, amain, telefon, schicht_vk, gewerk FROM master_responsible ORDER BY name")?;
        let rows = stmt.query_map([], |r| {
            Ok(crate::domain::Responsible {
                name: r.get(0)?,
                amain: r.get(1)?,
                telefon: r.get(2)?,
                schicht_vk: r.get(3)?,
                gewerk: r.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows { out.push(r?); }
        Ok(out)
    }
    pub fn update_responsible_details(&self, name: &str, amain: &str, telefon: &str, schicht_vk: &str, gewerk: &str) -> Result<()> {
        self.conn.execute("UPDATE master_responsible SET amain=?2, telefon=?3, schicht_vk=?4, gewerk=?5 WHERE name=?1", params![name, amain, telefon, schicht_vk, gewerk])?;
        Ok(())
    }

    pub fn delete_plant(&self, name: &str) -> Result<()> {
        self.conn.execute("DELETE FROM master_plants WHERE name=?1", params![name])?;
        Ok(())
    }
    pub fn rename_plant(&self, old: &str, new: &str) -> Result<()> {
        self.conn.execute("UPDATE master_plants SET name=?2 WHERE name=?1", params![old, new])?;
        self.conn.execute("UPDATE entries SET plant=?2 WHERE plant=?1", params![old, new])?;
        Ok(())
    }
    pub fn delete_responsible(&self, name: &str) -> Result<()> {
        self.conn.execute("DELETE FROM master_responsible WHERE name=?1", params![name])?;
        Ok(())
    }
    pub fn rename_responsible(&self, old: &str, new: &str) -> Result<()> {
        self.conn.execute("UPDATE master_responsible SET name=?2 WHERE name=?1", params![old, new])?;
        self.conn.execute("UPDATE entries SET responsible=?2 WHERE responsible=?1", params![old, new])?;
        Ok(())
    }

    pub fn list_statuses(&self) -> Result<Vec<(String, i32)>> {
        let mut stmt = self.conn.prepare("SELECT name, weight FROM master_statuses ORDER BY weight ASC, name ASC")?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        let mut out = Vec::new();
        for r in rows { out.push(r?); }
        Ok(out)
    }
    pub fn add_status(&self, name: &str, weight: i32) -> Result<()> {
        self.conn.execute("INSERT OR IGNORE INTO master_statuses (name, weight) VALUES (?1, ?2)", params![name, weight])?;
        Ok(())
    }
    pub fn delete_status(&self, name: &str) -> Result<()> {
        self.conn.execute("DELETE FROM master_statuses WHERE name=?1", params![name])?;
        Ok(())
    }
    pub fn rename_status(&self, old: &str, new: &str) -> Result<()> {
        self.conn.execute("UPDATE master_statuses SET name=?2 WHERE name=?1", params![old, new])?;
        self.conn.execute("UPDATE entries SET status=?2 WHERE status=?1", params![old, new])?;
        Ok(())
    }
    pub fn set_status_weight(&self, name: &str, weight: i32) -> Result<()> {
        self.conn.execute("UPDATE master_statuses SET weight=?2 WHERE name=?1", params![name, weight])?;
        Ok(())
    }

    pub fn log_event(&self, event_type: &str, user: &str, details: &str, related_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (id, timestamp, event_type, user, details, related_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![uuid::Uuid::new_v4().to_string(), chrono::Local::now().naive_local().to_string(), event_type, user, details, related_id],
        )?;
        Ok(())
    }
    pub fn list_events(&self, limit: usize) -> Result<Vec<crate::domain::Event>> {
        let mut stmt = self.conn.prepare("SELECT id, timestamp, event_type, user, details, related_id FROM events ORDER BY timestamp DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit as i64], |r| {
            Ok(crate::domain::Event {
                id: r.get(0)?,
                timestamp: r.get::<_,String>(1)?.parse().unwrap_or_else(|_| chrono::Local::now().naive_local()),
                event_type: r.get(2)?,
                user: r.get(3)?,
                details: r.get(4)?,
                related_id: r.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for e in rows { out.push(e?); }
        Ok(out)
    }

    pub fn list_gewerke(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT name FROM master_gewerke ORDER BY name")?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        let mut out = Vec::new();
        for r in rows { out.push(r?); }
        Ok(out)
    }
    pub fn add_gewerk(&self, name: &str) -> Result<()> {
        self.conn.execute("INSERT OR IGNORE INTO master_gewerke (name) VALUES (?1)", params![name])?;
        Ok(())
    }
    pub fn delete_gewerk(&self, name: &str) -> Result<()> {
        self.conn.execute("DELETE FROM master_gewerke WHERE name=?1", params![name])?;
        Ok(())
    }
    pub fn rename_gewerk(&self, old: &str, new: &str) -> Result<()> {
        self.conn.execute("UPDATE master_gewerke SET name=?2 WHERE name=?1", params![old, new])?;
        self.conn.execute("UPDATE entries SET gewerk=?2 WHERE gewerk=?1", params![old, new])?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<String> {
        self.conn.query_row("SELECT value FROM settings WHERE key=?1", params![key], |r| r.get(0))
            .or_else(|_| Ok(String::new()))
    }
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute("INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=?2", params![key, value])?;
        Ok(())
    }

    pub fn count_entries(&self, archived: bool) -> Result<i64> {
        self.conn.query_row("SELECT COUNT(*) FROM entries WHERE archived=?1", params![archived as i32], |r| r.get(0))
    }
}

fn row_to_entry(r: &rusqlite::Row) -> rusqlite::Result<Entry> {
    let date_str: String = r.get(1)?;
    let time_str: String = r.get(2)?;
    let last_shift: Option<String> = r.get(4)?;
    let last_editor: String = r.get(5)?;
    let created: String = r.get(21)?;
    let updated: String = r.get(22)?;
    Ok(Entry {
        id: r.get(0)?,
        date: date_str.parse().unwrap_or_else(|_| chrono::Local::now().date_naive()),
        time: time_str,
        shift: Shift::from_str(&r.get::<_,String>(3)?),
        last_shift: last_shift.map(|s| Shift::from_str(&s)),
        last_editor,
        status: r.get(6)?,
        priority: r.get(7)?,
        plant: r.get(8)?,
        sap_order: r.get(9)?,
        fault: r.get(10)?,
        responsible: r.get(11)?,
        gewerk: r.get(12)?,
        wartung: r.get::<_,Option<i32>>(13).unwrap_or(Some(0)).unwrap_or(0) != 0,
        kommentar: r.get(14)?,
        handed_over_to: r.get(15)?,
        downtime: r.get(16)?,
        action: r.get(17)?,
        images: r.get(18)?,
        transferred: r.get::<_,i32>(19)? != 0,
        archived: r.get::<_,i32>(20)? != 0,
        created_at: created.parse().unwrap_or_else(|_| chrono::Local::now().naive_local()),
        updated_at: updated.parse().unwrap_or_else(|_| chrono::Local::now().naive_local()),
        created_by: r.get(23)?,
    })
}

