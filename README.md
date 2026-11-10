# Schichtbuch

Digitales Schichtbuch als Ersatz für die Excel-Vorlage `Tagesbericht.xlsx`.
Entwickelt in Rust, speichert in SQLite (Datei auf Netzlaufwerk) oder – über
die Storage-Abstraktion – in SharePoint-Listen.

## Anforderungen → Umsetzung

| Anforderung | Umsetzung |
|---|---|
| Schreiben/Lesen durch mehrere Nutzer gleichzeitig | SQLite im WAL-Modus mit `busy_timeout=30s`; Schreibvorgänge in Transaktionen. Für SharePoint: Storage-Trait (siehe `src/storage.rs`) kann durch Graph-API-Backend ersetzt werden. |
| Kein Server, nur Netzlaufwerk/SharePoint | Single-File-Datenbank (`schichtbuch.db`) auf Freigabe; keine Server-Software nötig. GUI/CLI laufen lokal auf jedem Client. |
| Schichtübergabe/-übernahme per Knopfdruck | `Handover`-Datensatz + Befehl/GUI-Button "Schichtübergabe durchführen": archiviert erledigte Aufträge, übernimmt offene. |
| Arbeitsaufträge nummerisch priorisierbar | Feld `priority: Option<i32>`; Sortierung in "Prio offen" und Current-Shift nach Prio aufsteigend. |
| ~50.000 Einträge/Jahr | SQLite mit Indizes (`date,shift`, `archived`, `status`, `priority`); Suche über LIKE mit Index-Unterstützung. |
| Schichten Früh/Spät/Nacht, 4-Schicht | Enum `Shift {Frueh, Spaet, Nacht, Tag}`. |
| Erledigte Aufträge bei Übergabe archivieren, aber durchsuchbar | `archived`-Flag; `archive_completed()`; `search_archive()` (Volltext über Anlage, Störmeldung, Maßnahme, SAP, Verantwortlich). |
| Vorlage Excel | Importer (`src/import.rs`, calamine) liest `Tagesbericht.xlsx`: Stammdaten + Historie + Prio offen. |

## Datenmodell

- `Entry`: id, date, shift, status, priority, plant (Anlage), sap_order, fault (Störmeldung), responsible, handed_over_to, downtime, action (Maßnahme), images, transferred, archived, timestamps, created_by.
- `Handover`: from_shift, to_shift, date, performed_by, performed_at, archived_count, carried_count.
- Stammdaten: plants, responsible, statuses, shifts.

## Build

```sh
cargo build --release
```

## Nutzung (CLI)

```sh
# DB anlegen / öffnen
schichtbuch init /pfad/zur/freigabe/schichtbuch.db

# Excel-Vorlage importieren
schichtbuch import Tagesbericht.xlsx --db /pfad/.../schichtbuch.db

# Statistik
schichtbuch stats --db ...

# Offene Prioritäten
schichtbuch prio --db ...

# Archiv durchsuchen
schichtbuch search --db ... "Presse"

# Neuer Eintrag
schichtbuch add --db ... --date 2026-11-10 --shift Früh --plant "Presse 1" --status offen --priority 3

# Status / Priorität ändern
schichtbuch update-status --db ... <id> erledigt
schichtbuch set-priority --db ... <id> 1

# Erledigte archivieren
schichtbuch archive --db ... --date 2026-11-10 --shift Früh

# Schichtübergabe (Knopfdruck-Equivalent)
schichtbuch handover --db ... --from Früh --to Spät --date 2026-11-10 --user "Name"

# GUI starten
schichtbuch gui --db /pfad/.../schichtbuch.db
```

## Multi-User / Netzlaufwerk

- DB-Pfad zeigt auf die Freigabe (z. B. `\\server\share\schichtbuch.db` oder gemountet unter Linux).
- WAL-Modus erlaubt gleichzeitige Leser und einen Schreiber; Schreibkonflikte werden durch `busy_timeout` (30 s) serialisiert.
- Empfehlung: Freigabe mit SMB/CIFS und Dateisperren; bei SharePoint stattdessen Storage-Backend auf Graph API umstellen (Interface in `storage.rs`).

## Archiv & Suche

Erledigte Einträge werden bei der Schichtübergabe (oder manuell) archiviert (`archived=1`) und bleiben in derselben Tabelle durchsuchbar (`search_archive`).

## Projektstruktur

```
src/
  main.rs      CLI + Befehle
  domain.rs    Domänenmodell
  storage.rs   SQLite-Storage (Multi-User)
  import.rs    Excel-Import (calamine)
  gui.rs       egui-Desktop-GUI
```
