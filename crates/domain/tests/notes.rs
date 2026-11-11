mod mem;

use domain::{Draft, Error, Event, Id, Item, Kind, Notes, Patch, image, item, rank};
use mem::{Log, full};

fn app() -> (Notes, Log) {
    let (n, log, _) = full();
    (n, log)
}

fn titles(n: &Notes) -> Vec<String> {
    n.list().unwrap().into_iter().map(|n| n.title).collect()
}

#[test]
fn creates_each_kind() {
    let (n, log) = app();
    n.add(Draft::new(Kind::Sticky).body("porta 7341")).unwrap();
    n.add(Draft::new(Kind::Text).title("Ideia").body("grid simples")).unwrap();
    n.add(Draft::new(Kind::Todo).title("Mercado").item("café")).unwrap();

    assert_eq!(n.list().unwrap().len(), 3);
    assert!(log.lock().unwrap().iter().all(|e| matches!(e, Event::Created { .. })));
}

#[test]
fn newest_note_comes_first() {
    let (n, _) = app();
    n.add(Draft::new(Kind::Text).title("velha")).unwrap();
    n.add(Draft::new(Kind::Text).title("nova")).unwrap();

    assert_eq!(titles(&n), ["nova", "velha"]);
}

#[test]
fn each_kind_keeps_its_shape() {
    let (n, _) = app();
    let long = "a".repeat(501);

    assert_eq!(
        n.add(Draft::new(Kind::Sticky).title("x")).unwrap_err(),
        Error::Invalid("sticky não tem título")
    );
    assert!(n.add(Draft::new(Kind::Sticky).body(long)).is_err());
    assert!(n.add(Draft::new(Kind::Text).item("x")).is_err());
    assert!(n.add(Draft::new(Kind::Todo).body("x")).is_err());
    assert!(n.add(Draft::new(Kind::Todo).item("  ")).is_err());
    assert!(n.list().unwrap().is_empty());
}

#[test]
fn typing_saves_and_notifies() {
    let (n, log) = app();
    let a = n.add(Draft::new(Kind::Text).title("Ideia")).unwrap();

    let b = n.edit(a.id, Patch { body: Some("grid simples".into()), ..Default::default() }).unwrap();

    assert_eq!(n.get(a.id).unwrap().body, "grid simples");
    assert!(b.updated > a.updated);
    assert_eq!(log.lock().unwrap().last(), Some(&Event::Updated { note: b }));
}

#[test]
fn drag_drops_between_neighbors() {
    let (n, _) = app();
    n.add(Draft::new(Kind::Text).title("c")).unwrap();
    n.add(Draft::new(Kind::Text).title("b")).unwrap();
    let a = n.add(Draft::new(Kind::Text).title("a")).unwrap();
    let v = n.list().unwrap();

    let r = rank::between(Some(v[1].rank), Some(v[2].rank));
    n.edit(a.id, Patch { rank: Some(r), ..Default::default() }).unwrap();

    assert_eq!(titles(&n), ["b", "a", "c"]);
}

#[test]
fn todo_items_flow() {
    let (n, _) = app();
    let t = n.add(Draft::new(Kind::Todo).title("Mercado")).unwrap();

    let t = n.push(t.id, Item::new("café")).unwrap();
    let t = n.push(t.id, Item::new("pão")).unwrap();
    let cafe = t.items[0].id;
    let t = n.mark(t.id, cafe, item::Patch { done: Some(true), ..Default::default() }).unwrap();
    assert!(t.items[0].done);

    let t = n.pull(t.id, cafe).unwrap();
    assert_eq!(t.items.len(), 1);
    assert_eq!(t.items[0].text, "pão");
    assert_eq!(n.pull(t.id, cafe).unwrap_err(), Error::NoItem);
}

#[test]
fn only_todo_takes_items() {
    let (n, _) = app();
    let s = n.add(Draft::new(Kind::Sticky).body("oi")).unwrap();

    assert!(n.push(s.id, Item::new("x")).is_err());
}

#[test]
fn bad_edit_changes_nothing() {
    let (n, log) = app();
    let s = n.add(Draft::new(Kind::Sticky).body("oi")).unwrap();
    let before = log.lock().unwrap().len();

    let e = n.edit(s.id, Patch { title: Some("x".into()), ..Default::default() });

    assert!(e.is_err());
    assert_eq!(n.get(s.id).unwrap(), s);
    assert_eq!(log.lock().unwrap().len(), before);
}

#[test]
fn removing_notifies_once() {
    let (n, log) = app();
    let s = n.add(Draft::new(Kind::Sticky).body("oi")).unwrap();

    n.remove(s.id).unwrap();

    assert_eq!(log.lock().unwrap().last(), Some(&Event::Deleted { id: s.id }));
    assert_eq!(n.get(s.id).unwrap_err(), Error::Missing);
    assert_eq!(n.remove(s.id).unwrap_err(), Error::Missing);
}

#[test]
fn pinned_notes_come_first() {
    let (n, _) = app();
    let a = n.add(Draft::new(Kind::Text).title("a")).unwrap();
    n.add(Draft::new(Kind::Text).title("b")).unwrap();
    n.add(Draft::new(Kind::Text).title("c")).unwrap();

    n.edit(a.id, Patch { pinned: Some(true), ..Default::default() }).unwrap();

    assert_eq!(titles(&n), ["a", "c", "b"]);
}

#[test]
fn search_ignores_case_and_accents() {
    let (n, _) = app();
    n.add(Draft::new(Kind::Sticky).body("comprar filtro de Café")).unwrap();
    n.add(Draft::new(Kind::Text).title("Reunião ABRAZ")).unwrap();

    assert_eq!(n.find("cafe").unwrap().len(), 1);
    assert_eq!(n.find("REUNIAO").unwrap()[0].title, "Reunião ABRAZ");
}

#[test]
fn search_needs_every_term() {
    let (n, _) = app();
    n.add(Draft::new(Kind::Text).title("Mercado").body("pão e leite")).unwrap();
    n.add(Draft::new(Kind::Text).title("Padaria").body("pão")).unwrap();

    let t: Vec<_> = n.find("pao mercado").unwrap().into_iter().map(|n| n.title).collect();
    assert_eq!(t, ["Mercado"]);
}

#[test]
fn search_reads_todo_items() {
    let (n, _) = app();
    n.add(Draft::new(Kind::Todo).title("Viagem").item("passaporte")).unwrap();

    assert_eq!(n.find("passa").unwrap()[0].title, "Viagem");
    assert!(n.find("mala").unwrap().is_empty());
}

#[test]
fn board_holds_a_scene() {
    let (n, _) = app();
    let b = n.add(Draft::new(Kind::Board)).unwrap();

    let scene = r#"{"v":1,"els":[{"id":"a","t":"rect","c":0,"x":0,"y":0,"w":40,"h":20}]}"#;
    assert_eq!(n.edit(b.id, Patch { body: Some(scene.into()), ..Default::default() }).unwrap().body, scene);
    assert_eq!(
        n.edit(b.id, Patch { body: Some("rabisco".into()), ..Default::default() }).unwrap_err(),
        Error::Invalid("cena do board inválida")
    );
    assert!(n.add(Draft::new(Kind::Board).item("x")).is_err());
    assert!(n.add(Draft::new(Kind::Board).body(format!(r#"{{"els":[],"pad":"{}"}}"#, "a".repeat(1_000_000)))).is_err());
    assert_eq!(n.attach(b.id, "image/png", PNG, 640, 480).unwrap().images.len(), 1);
}

#[test]
fn search_reads_board_texts() {
    let (n, _) = app();
    let scene = r#"{"v":1,"els":[{"id":"a","t":"text","c":0,"x":0,"y":0,"s":"Fluxo de login"}]}"#;
    n.add(Draft::new(Kind::Board).title("Arquitetura").body(scene)).unwrap();

    assert_eq!(n.find("login").unwrap()[0].title, "Arquitetura");
    assert!(n.find("els").unwrap().is_empty());
}

#[test]
fn empty_search_lists_everything() {
    let (n, _) = app();
    n.add(Draft::new(Kind::Sticky).body("x")).unwrap();
    n.add(Draft::new(Kind::Sticky).body("y")).unwrap();

    assert_eq!(n.find("  ").unwrap().len(), 2);
}

const PNG: &[u8] = b"\x89PNG fake";

#[test]
fn image_goes_into_the_note_and_comes_back() {
    let (n, log, files) = full();
    let t = n.add(Draft::new(Kind::Text).title("Ideia")).unwrap();

    let t = n.attach(t.id, "image/webp", PNG, 1200, 750).unwrap();

    let img = &t.images[0];
    assert_eq!((img.w, img.h, img.size), (1200, 750, PNG.len() as u64));
    assert_eq!(n.image(img.id).unwrap(), ("image/webp".into(), PNG.to_vec()));
    assert_eq!(files.0.lock().unwrap().len(), 1);
    assert!(matches!(log.lock().unwrap().last(), Some(Event::Updated { .. })));
}

#[test]
fn removing_image_frees_the_file() {
    let (n, _, files) = full();
    let t = n.add(Draft::new(Kind::Todo).title("Mercado")).unwrap();
    let t = n.attach(t.id, "image/png", PNG, 0, 0).unwrap();

    let t = n.detach(t.id, t.images[0].id).unwrap();

    assert!(t.images.is_empty());
    assert!(files.0.lock().unwrap().is_empty());
    assert_eq!(n.detach(t.id, Id::new()).unwrap_err(), Error::NoImage);
}

#[test]
fn only_real_images_are_accepted() {
    let (n, _, files) = full();
    let t = n.add(Draft::new(Kind::Text)).unwrap();

    assert!(n.attach(t.id, "image/svg+xml", b"<svg/>", 0, 0).is_err());
    assert!(n.attach(t.id, "application/pdf", PNG, 0, 0).is_err());
    assert!(n.attach(t.id, "image/png", &[], 0, 0).is_err());
    assert!(n.attach(t.id, "image/png", &vec![0; image::MAX + 1], 0, 0).is_err());
    assert!(files.0.lock().unwrap().is_empty());
}

#[test]
fn sticky_holds_a_single_photo() {
    let (n, _, files) = full();
    let s = n.add(Draft::new(Kind::Sticky).body("polaroid")).unwrap();
    n.attach(s.id, "image/jpeg", PNG, 0, 0).unwrap();

    let e = n.attach(s.id, "image/jpeg", PNG, 0, 0);

    assert_eq!(e.unwrap_err(), Error::Invalid("sticky aceita uma imagem"));
    assert_eq!(n.get(s.id).unwrap().images.len(), 1);
    assert_eq!(files.0.lock().unwrap().len(), 1);
}

#[test]
fn deleting_note_deletes_its_images() {
    let (n, _, files) = full();
    let t = n.add(Draft::new(Kind::Text)).unwrap();
    n.attach(t.id, "image/png", PNG, 0, 0).unwrap();
    n.attach(t.id, "image/gif", PNG, 0, 0).unwrap();

    n.remove(t.id).unwrap();

    assert!(files.0.lock().unwrap().is_empty());
}
