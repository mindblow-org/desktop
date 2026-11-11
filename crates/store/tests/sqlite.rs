use std::{path::PathBuf, sync::Arc};

use domain::{Bus, Draft, Id, Kind, Notes, item};
use store::Sqlite;

struct Tmp(PathBuf);

impl Tmp {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("mindblow-{}", Id::new())).join("notes.db"))
    }

    fn notes(&self) -> Notes {
        let db = Arc::new(Sqlite::open(&self.0).unwrap());
        Notes::new(db.clone(), db, Arc::new(Bus::default()))
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        if let Some(d) = self.0.parent() {
            let _ = std::fs::remove_dir_all(d);
        }
    }
}

#[test]
fn notes_survive_restart() {
    let tmp = Tmp::new();
    let saved = {
        let n = tmp.notes();
        n.add(Draft::new(Kind::Text).title("Ideia").body("grid simples")).unwrap();
        n.add(Draft::new(Kind::Sticky).body("porta 7341")).unwrap();
        n.list().unwrap()
    };

    assert_eq!(tmp.notes().list().unwrap(), saved);
}

#[test]
fn todo_items_keep_state() {
    let tmp = Tmp::new();
    let n = tmp.notes();
    let t = n.add(Draft::new(Kind::Todo).title("Mercado").item("café").item("pão")).unwrap();
    n.mark(t.id, t.items[1].id, item::Patch { done: Some(true), ..Default::default() }).unwrap();
    drop(n);

    let t = tmp.notes().get(t.id).unwrap();
    assert_eq!(t.items.iter().map(|i| (i.text.as_str(), i.done)).collect::<Vec<_>>(), [("café", false), ("pão", true)]);
}

#[test]
fn removed_note_stays_gone() {
    let tmp = Tmp::new();
    let n = tmp.notes();
    let s = n.add(Draft::new(Kind::Sticky).body("oi")).unwrap();
    n.remove(s.id).unwrap();
    drop(n);

    assert!(tmp.notes().list().unwrap().is_empty());
}

#[test]
fn new_notes_go_above_saved_ones() {
    let tmp = Tmp::new();
    tmp.notes().add(Draft::new(Kind::Text).title("antiga")).unwrap();

    let n = tmp.notes();
    n.add(Draft::new(Kind::Text).title("nova")).unwrap();

    let t: Vec<_> = n.list().unwrap().into_iter().map(|n| n.title).collect();
    assert_eq!(t, ["nova", "antiga"]);
}

#[test]
fn pin_survives_restart() {
    let tmp = Tmp::new();
    let n = tmp.notes();
    let s = n.add(Draft::new(Kind::Sticky).body("porta 7341").pin()).unwrap();
    drop(n);

    assert!(tmp.notes().get(s.id).unwrap().pinned);
}

#[test]
fn old_database_keeps_notes_after_upgrade() {
    let tmp = Tmp::new();
    std::fs::create_dir_all(tmp.0.parent().unwrap()).unwrap();
    let old = rusqlite::Connection::open(&tmp.0).unwrap();
    old.execute_batch(
        "create table notes (id text primary key, kind text not null, title text not null default '',
         body text not null default '', items text not null default '[]', rank real not null,
         created integer not null, updated integer not null);",
    )
    .unwrap();
    let id = Id::new();
    old.execute(
        "insert into notes (id, kind, title, body, items, rank, created, updated) values (?1, 'text', 'Ideia', 'grid', '[]', 0, 1, 1)",
        [id.to_string()],
    )
    .unwrap();
    drop(old);

    let n = tmp.notes();
    let note = n.get(id).unwrap();
    assert_eq!((note.title.as_str(), note.pinned), ("Ideia", false));
    n.edit(id, domain::Patch { pinned: Some(true), ..Default::default() }).unwrap();
    assert!(n.get(id).unwrap().pinned);
}

#[test]
fn images_survive_restart_and_leave_with_the_note() {
    let tmp = Tmp::new();
    let n = tmp.notes();
    let t = n.add(Draft::new(Kind::Text).title("Ideia")).unwrap();
    let t = n.attach(t.id, "image/webp", b"RIFF fake webp", 1200, 750).unwrap();
    let img = t.images[0].id;
    drop(n);

    let n = tmp.notes();
    assert_eq!(n.get(t.id).unwrap().images[0].w, 1200);
    assert_eq!(n.image(img).unwrap().1, b"RIFF fake webp");

    n.remove(t.id).unwrap();
    assert!(n.image(img).is_err());
}
