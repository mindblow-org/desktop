mod mem;

use std::{thread::sleep, time::Duration};

use domain::{Color, Draft, Event, Shot, Item, Kind, Note, Notes, Patch, item, merge};
use mem::full;

/// Leva tudo que `a` tem para `b`.
fn send(a: &Notes, b: &Notes) -> Vec<domain::Id> {
    let f = a.feed(0, 1000).unwrap();
    b.take(f.notes, f.gone).unwrap().need
}

/// Os dois lados trocam até não sobrar diferença.
fn both(a: &Notes, b: &Notes) {
    send(a, b);
    send(b, a);
}

fn same(a: &Notes, b: &Notes) {
    let (x, y) = (a.list().unwrap(), b.list().unwrap());
    assert_eq!(x, y);
    assert_eq!(x.iter().map(|n| &n.clock).collect::<Vec<_>>(), y.iter().map(|n| &n.clock).collect::<Vec<_>>());
}

fn tick() {
    sleep(Duration::from_millis(3));
}

#[test]
fn note_made_offline_shows_up_on_the_other_side() {
    let (a, _, _) = full();
    let (b, log, _) = full();
    let n = a.add(Draft::new(Kind::Todo).title("Mercado").item("café")).unwrap();

    send(&a, &b);

    assert_eq!(b.get(n.id).unwrap(), n);
    assert!(matches!(log.lock().unwrap()[..], [Event::Created { .. }]));
}

#[test]
fn edits_to_different_fields_both_survive() {
    let (a, _, _) = full();
    let (b, _, _) = full();
    let n = a.add(Draft::new(Kind::Text).title("Ideia").body("grid")).unwrap();
    send(&a, &b);

    a.edit(n.id, Patch { title: Some("Ideia boa".into()), ..Default::default() }).unwrap();
    b.edit(n.id, Patch { body: Some("grid simples".into()), pinned: Some(true), ..Default::default() }).unwrap();
    both(&a, &b);

    let m = a.get(n.id).unwrap();
    assert_eq!((m.title.as_str(), m.body.as_str(), m.pinned), ("Ideia boa", "grid simples", true));
    same(&a, &b);
}

#[test]
fn same_field_keeps_the_latest() {
    let (a, _, _) = full();
    let (b, _, _) = full();
    let n = a.add(Draft::new(Kind::Sticky).body("v1")).unwrap();
    send(&a, &b);

    a.edit(n.id, Patch { body: Some("do notebook".into()), ..Default::default() }).unwrap();
    tick();
    b.edit(n.id, Patch { body: Some("do celular".into()), ..Default::default() }).unwrap();
    both(&a, &b);

    assert_eq!(a.get(n.id).unwrap().body, "do celular");
    same(&a, &b);
}

#[test]
fn todo_items_merge_one_by_one() {
    let (a, _, _) = full();
    let (b, _, _) = full();
    let n = a.add(Draft::new(Kind::Todo).title("Mercado").item("café").item("pão")).unwrap();
    let (cafe, pao) = (n.items[0].id, n.items[1].id);
    send(&a, &b);

    a.push(n.id, Item::new("leite")).unwrap();
    a.pull(n.id, pao).unwrap();
    b.push(n.id, Item::new("ovos")).unwrap();
    b.mark(n.id, cafe, item::Patch { done: Some(true), ..Default::default() }).unwrap();
    both(&a, &b);

    let m = a.get(n.id).unwrap();
    let got: Vec<_> = m.items.iter().map(|i| (i.text.as_str(), i.done)).collect();
    assert!(got.contains(&("café", true)));
    assert!(got.contains(&("leite", false)));
    assert!(got.contains(&("ovos", false)));
    assert_eq!(got.len(), 3);
    same(&a, &b);
}

#[test]
fn whole_list_patch_only_stamps_what_changed() {
    let (a, _, _) = full();
    let (b, _, _) = full();
    let n = a.add(Draft::new(Kind::Todo).title("Mercado").item("café").item("pão")).unwrap();
    send(&a, &b);

    let mut items = n.items.clone();
    items[0].text = "café forte".into();
    a.edit(n.id, Patch { items: Some(items), ..Default::default() }).unwrap();
    tick();
    b.mark(n.id, n.items[1].id, item::Patch { done: Some(true), ..Default::default() }).unwrap();
    both(&a, &b);

    let m = b.get(n.id).unwrap();
    assert_eq!((m.items[0].text.as_str(), m.items[1].done), ("café forte", true));
    same(&a, &b);
}

#[test]
fn delete_wins_over_older_edit() {
    let (a, _, _) = full();
    let (b, log, _) = full();
    let n = a.add(Draft::new(Kind::Sticky).body("oi")).unwrap();
    send(&a, &b);

    b.edit(n.id, Patch { body: Some("oi!".into()), ..Default::default() }).unwrap();
    tick();
    a.remove(n.id).unwrap();
    both(&a, &b);

    assert!(b.list().unwrap().is_empty());
    assert!(a.list().unwrap().is_empty());
    assert_eq!(log.lock().unwrap().last(), Some(&Event::Deleted { id: n.id }));
}

#[test]
fn edit_after_delete_brings_the_note_back() {
    let (a, _, _) = full();
    let (b, _, _) = full();
    let n = a.add(Draft::new(Kind::Sticky).body("oi")).unwrap();
    send(&a, &b);

    a.remove(n.id).unwrap();
    tick();
    b.edit(n.id, Patch { body: Some("ainda preciso".into()), ..Default::default() }).unwrap();
    both(&a, &b);

    assert_eq!(a.get(n.id).unwrap().body, "ainda preciso");
    same(&a, &b);
}

#[test]
fn delete_reaches_a_node_that_never_saw_the_note() {
    let (a, _, _) = full();
    let (b, _, _) = full();
    let (c, _, _) = full();
    let n = a.add(Draft::new(Kind::Sticky).body("oi")).unwrap();
    send(&a, &c);
    a.remove(n.id).unwrap();

    send(&a, &b);
    send(&c, &b);

    assert!(b.list().unwrap().is_empty());
}

#[test]
fn what_comes_back_does_not_echo() {
    let (a, log, _) = full();
    let (b, _, _) = full();
    let n = a.add(Draft::new(Kind::Text).title("Ideia")).unwrap();
    a.edit(n.id, Patch { body: Some("x".into()), ..Default::default() }).unwrap();
    send(&a, &b);
    let before = log.lock().unwrap().len();

    let f = b.feed(0, 1000).unwrap();
    let t = a.take(f.notes, f.gone).unwrap();

    assert_eq!(t.from, t.to);
    assert_eq!(log.lock().unwrap().len(), before);
}

#[test]
fn feed_comes_in_pages() {
    let (a, _, _) = full();
    for i in 0..5 {
        a.add(Draft::new(Kind::Sticky).body(format!("{i}"))).unwrap();
    }

    let p = a.feed(0, 3).unwrap();
    let q = a.feed(p.seq, 3).unwrap();

    assert_eq!((p.notes.len(), p.more), (3, true));
    assert_eq!((q.notes.len(), q.more), (2, false));
    assert_eq!(a.feed(q.seq, 3).unwrap().notes.len(), 0);
}

const PNG: &[u8] = b"\x89PNG fake";

#[test]
fn images_arrive_after_the_note() {
    let (a, _, _) = full();
    let (b, _, files) = full();
    let n = a.add(Draft::new(Kind::Text).title("Foto")).unwrap();
    let n = a.attach(n.id, "image/png", PNG, 10, 10).unwrap();
    let img = n.images[0].id;

    let need = send(&a, &b);
    assert_eq!(need, [img]);
    assert_eq!(b.lack().unwrap(), [img]);

    let (mime, data) = a.image(img).unwrap();
    b.adopt(img, &mime, &data).unwrap();
    assert!(b.lack().unwrap().is_empty());
    assert_eq!(files.0.lock().unwrap().len(), 1);
}

#[test]
fn images_added_on_both_sides_are_kept() {
    let (a, _, _) = full();
    let (b, _, fb) = full();
    let n = a.add(Draft::new(Kind::Text).title("Fotos")).unwrap();
    send(&a, &b);

    a.attach(n.id, "image/png", PNG, 0, 0).unwrap();
    let mine = b.attach(n.id, "image/gif", PNG, 0, 0).unwrap().images[0].id;
    both(&a, &b);

    assert_eq!(a.get(n.id).unwrap().images.len(), 2);
    same(&a, &b);
    assert!(fb.0.lock().unwrap().contains_key(&mine));
}

#[test]
fn merge_gives_the_same_note_either_way() {
    let (a, _, _) = full();
    let (b, _, _) = full();
    let n = a.add(Draft::new(Kind::Todo).title("t").item("1").item("2")).unwrap();
    send(&a, &b);
    a.edit(n.id, Patch { title: Some("a".into()), rank: Some(5.0), ..Default::default() }).unwrap();
    a.push(n.id, Item::new("3")).unwrap();
    b.edit(n.id, Patch { title: Some("b".into()), pinned: Some(true), ..Default::default() }).unwrap();
    b.pull(n.id, n.items[0].id).unwrap();

    let (x, y): (Note, Note) = (a.get(n.id).unwrap(), b.get(n.id).unwrap());

    assert_eq!(merge(&x, &y), merge(&y, &x));
    assert_eq!(merge(&x, &x), x);
    let m = merge(&x, &y);
    assert_eq!(merge(&m, &y), m);
}

#[test]
fn color_on_one_side_and_text_on_the_other_both_survive() {
    let (a, _, _) = full();
    let (b, _, _) = full();
    let n = a.add(Draft::new(Kind::Sticky).body("porta")).unwrap();
    send(&a, &b);

    a.edit(n.id, Patch { color: Some(Color::Sky), ..Default::default() }).unwrap();
    tick();
    b.edit(n.id, Patch { body: Some("porta 7341".into()), ..Default::default() }).unwrap();
    both(&a, &b);

    let m = a.get(n.id).unwrap();
    assert_eq!((m.color, m.body.as_str()), (Color::Sky, "porta 7341"));
    same(&a, &b);
}

#[test]
fn old_mindblow_without_color_does_not_wash_it_out() {
    let (a, _, _) = full();
    let n = a.add(Draft::new(Kind::Sticky).body("porta").color(Color::Pink)).unwrap();
    let mut old = serde_json::to_value(&a.feed(0, 10).unwrap().notes[0]).unwrap();
    old["note"].as_object_mut().unwrap().remove("color");
    old["clock"].as_object_mut().unwrap().remove("color");
    old["note"]["body"] = "porta 7341".into();
    old["note"]["updated"] = (n.updated + 10).into();
    old["clock"]["body"] = (n.updated + 10).into();

    a.take(vec![serde_json::from_value::<Shot>(old).unwrap()], Vec::new()).unwrap();

    let m = a.get(n.id).unwrap();
    assert_eq!((m.color, m.body.as_str()), (Color::Pink, "porta 7341"));
}
