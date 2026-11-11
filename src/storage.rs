use crate::domain::{Entry, Handover, Shift, Status};
use chrono::NaiveDate;
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct Store {
    pub conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        // multi-user friendly settings
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        conn.pragma_update(None, "busy_timeout", 30000)?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
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
            CREATE TABLE IF NOT EXISTS master_responsible (name TEXT PRIMARY KEY);
            CREATE TABLE IF NOT EXISTS master_statuses (name TEXT PRIMARY KEY);
            CREATE TABLE IF NOT EXISTS master_shifts (name TEXT PRIMARY KEY);

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL DEFAULT ''
            );
            "#,
        )?;
        // migration: add last_shift column if missing
        let _ = self.conn.execute("ALTER TABLE entries ADD COLUMN last_shift TEXT", []);
        let _ = self.conn.execute("ALTER TABLE entries ADD COLUMN last_editor TEXT", []);
        let _ = self.conn.execute("ALTER TABLE entries ADD COLUMN frozen INTEGER DEFAULT 0", []);
        let _ = self.conn.execute("ALTER TABLE entries ADD COLUMN time TEXT DEFAULT ''", []);
        Ok(())
    }

    pub fn insert_entry(&self, e: &Entry) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO entries (id,date,time,shift,last_shift,last_editor,frozen,status,priority,plant,sap_order,fault,responsible,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)"#,
            params![
                e.id,
                e.date.to_string(),
                e.time.clone(),
                e.shift.as_str(),
                e.last_shift.map(|s| s.as_str().to_string()),
                e.last_editor.clone(),
                e.frozen as i32,
                e.status.as_str(),
                e.priority,
                e.plant,
                e.sap_order,
                e.fault,
                e.responsible,
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
            r#"UPDATE entries SET date=?2,time=?3,shift=?4,last_shift=?5,last_editor=?6,frozen=?7,status=?8,priority=?9,plant=?10,sap_order=?11,fault=?12,responsible=?13,handed_over_to=?14,downtime=?15,action=?16,images=?17,transferred=?18,archived=?19,updated_at=?20,created_by=?21
               WHERE id=?1"#,
            params![
                e.id,
                e.date.to_string(),
                e.time.clone(),
                e.shift.as_str(),
                e.last_shift.map(|s| s.as_str().to_string()),
                e.last_editor.clone(),
                e.frozen as i32,
                e.status.as_str(),
                e.priority,
                e.plant,
                e.sap_order,
                e.fault,
                e.responsible,
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

    pub fn delete_entry(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM entries WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn get_entry(&self, id: &str) -> Result<Option<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,date,time,shift,last_shift,last_editor,frozen,status,priority,plant,sap_order,fault,responsible,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by FROM entries WHERE id=?1"
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
            "SELECT id,date,time,shift,last_shift,last_editor,frozen,status,priority,plant,sap_order,fault,responsible,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by FROM entries WHERE archived=?1 ORDER BY date DESC, priority ASC NULLS LAST, created_at DESC"
        )?;
        let rows = stmt.query_map(params![archived as i32], |r| row_to_entry(r))?;
        let mut out = Vec::new();
        for e in rows { out.push(e?); }
        Ok(out)
    }

    pub fn list_current_shift(&self, date: NaiveDate, shift: Shift) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,date,time,shift,last_shift,last_editor,frozen,status,priority,plant,sap_order,fault,responsible,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by FROM entries WHERE archived=0 AND date=?1 AND shift=?2 ORDER BY priority ASC NULLS LAST, created_at ASC"
        )?;
        let rows = stmt.query_map(params![date.to_string(), shift.as_str()], |r| row_to_entry(r))?;
        let mut out = Vec::new();
        for e in rows { out.push(e?); }
        Ok(out)
    }

    pub fn list_open_priorities(&self) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,date,time,shift,last_shift,last_editor,frozen,status,priority,plant,sap_order,fault,responsible,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by FROM entries WHERE archived=0 AND frozen=0 AND status<>'erledigt' ORDER BY priority ASC NULLS LAST, date ASC, created_at ASC"
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
        // select open entries in from shift
        let mut sql = "SELECT id,date,time,shift,last_shift,last_editor,frozen,status,priority,plant,sap_order,fault,responsible,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by FROM entries WHERE archived=0 AND frozen=0 AND status<>'erledigt' AND shift=?1".to_string();
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        args.push(Box::new(from.as_str().to_string()));
        if let Some(d) = date {
            sql.push_str(" AND date=?2");
            args.push(Box::new(d.to_string()));
        }
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(args.iter().map(|b| b.as_ref())), |r| row_to_entry(r))?;
        let mut entries: Vec<crate::domain::Entry> = Vec::new();
        for e in rows { entries.push(e?); }
        drop(stmt);

        let now = chrono::Local::now().naive_local();
        let mut carried = 0i64;
        for e in &entries {
            // freeze original
            self.conn.execute("UPDATE entries SET frozen=1, updated_at=?1 WHERE id=?2", rusqlite::params![now.to_string(), e.id])?;
            // create copy in target shift
            let mut copy = e.clone();
            copy.id = uuid::Uuid::new_v4().to_string();
            copy.shift = to;
            copy.frozen = false;
            copy.last_shift = Some(from);
            copy.last_editor = e.last_editor.clone();
            copy.created_at = now;
            copy.updated_at = now;
            // insert
            self.insert_entry(&copy)?;
            carried += 1;
        }
        Ok(carried)
    }

    pub fn search_archive(&self, query: &str) -> Result<Vec<Entry>> {
        let like = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id,date,time,shift,last_shift,last_editor,frozen,status,priority,plant,sap_order,fault,responsible,handed_over_to,downtime,action,images,transferred,archived,created_at,updated_at,created_by FROM entries WHERE archived=1 AND (plant LIKE ?1 OR fault LIKE ?1 OR action LIKE ?1 OR sap_order LIKE ?1 OR responsible LIKE ?1) ORDER BY date DESC, created_at DESC"
        )?;
        let rows = stmt.query_map(params![like], |r| row_to_entry(r))?;
        let mut out = Vec::new();
        for e in rows { out.push(e?); }
        Ok(out)
    }

    pub fn record_handover(&self, h: &Handover) -> Result<()> {
        self.conn.execute(
            "INSERT INTO handovers (id,from_shift,to_shift,date,performed_by,performed_at,archived_count,carried_count) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![h.id, h.from_shift.as_str(), h.to_shift.as_str(), h.date.to_string(), h.performed_by, h.performed_at.to_string(), h.archived_count, h.carried_count],
        )?;
        Ok(())
    }

    pub fn list_handovers(&self) -> Result<Vec<Handover>> {
        let mut stmt = self.conn.prepare("SELECT id,from_shift,to_shift,date,performed_by,performed_at,archived_count,carried_count FROM handovers ORDER BY performed_at DESC")?;
        let rows = stmt.query_map([], |r| {
            Ok(Handover {
                id: r.get(0)?,
                from_shift: Shift::from_str(&r.get::<_,String>(1)?),
                to_shift: Shift::from_str(&r.get::<_,String>(2)?),
                date: r.get::<_,String>(3)?.parse().unwrap_or_else(|_| chrono::Local::now().date_naive()),
                performed_by: r.get(4)?,
                performed_at: r.get::<_,String>(5)?.parse().unwrap_or_else(|_| chrono::Local::now().naive_local()),
                archived_count: r.get(6)?,
                carried_count: r.get(7)?,
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
    let frozen: i32 = r.get(6)?;
    let created: String = r.get(19)?;
    let updated: String = r.get(20)?;
    Ok(Entry {
        id: r.get(0)?,
        date: date_str.parse().unwrap_or_else(|_| chrono::Local::now().date_naive()),
        time: time_str,
        shift: Shift::from_str(&r.get::<_,String>(3)?),
        last_shift: last_shift.map(|s| Shift::from_str(&s)),
        last_editor,
        frozen: frozen != 0,
        status: Status::from_str(&r.get::<_,String>(7)?),
        priority: r.get(8)?,
        plant: r.get(9)?,
        sap_order: r.get(10)?,
        fault: r.get(11)?,
        responsible: r.get(12)?,
        handed_over_to: r.get(13)?,
        downtime: r.get(14)?,
        action: r.get(15)?,
        images: r.get(16)?,
        transferred: r.get::<_,i32>(17)? != 0,
        archived: r.get::<_,i32>(18)? != 0,
        created_at: created.parse().unwrap_or_else(|_| chrono::Local::now().naive_local()),
        updated_at: updated.parse().unwrap_or_else(|_| chrono::Local::now().naive_local()),
        created_by: r.get(21)?,
    })
}
