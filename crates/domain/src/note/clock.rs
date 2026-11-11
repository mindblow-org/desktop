use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use super::{Id, Image, Item, Note};

/// Por quanto tempo um item ou imagem removido fica lembrado. Um aparelho que passe
/// mais que isso offline pode trazer de volta o que foi removido nesse meio tempo.
const KEEP: i64 = 90 * 24 * 60 * 60 * 1000;

/// Quando cada parte da nota mudou pela última vez, para juntar versões de aparelhos
/// diferentes campo a campo. Item ou imagem que está aqui e não na nota foi removido.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Clock {
    #[serde(default)]
    pub title: i64,
    #[serde(default)]
    pub body: i64,
    #[serde(default)]
    pub pinned: i64,
    #[serde(default)]
    pub rank: i64,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub items: BTreeMap<Id, i64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub images: BTreeMap<Id, i64>,
}

impl Clock {
    /// Tudo carimbado no mesmo instante: nota nova, ou nota de antes do sync.
    pub fn at(n: &Note, t: i64) -> Self {
        Self {
            title: t,
            body: t,
            pinned: t,
            rank: t,
            items: n.items.iter().map(|i| (i.id, t)).collect(),
            images: n.images.iter().map(|i| (i.id, t)).collect(),
        }
    }
}

impl Note {
    /// Carimba com `t` o que mudou desde `old`.
    pub fn stamp(&mut self, old: &Note, t: i64) {
        let c = &mut self.clock;
        if self.title != old.title {
            c.title = t;
        }
        if self.body != old.body {
            c.body = t;
        }
        if self.pinned != old.pinned {
            c.pinned = t;
        }
        if self.rank != old.rank {
            c.rank = t;
        }
        touch(&mut c.items, &old.items, &self.items, |i: &Item| i.id, t);
        touch(&mut c.images, &old.images, &self.images, |i: &Image| i.id, t);
        self.updated = t;
    }
}

fn touch<T: PartialEq>(c: &mut BTreeMap<Id, i64>, old: &[T], new: &[T], id: impl Fn(&T) -> Id, t: i64) {
    let before: HashMap<Id, &T> = old.iter().map(|x| (id(x), x)).collect();
    for x in new {
        if before.get(&id(x)).is_none_or(|o| *o != x) {
            c.insert(id(x), t);
        }
    }
    let live: HashMap<Id, &T> = new.iter().map(|x| (id(x), x)).collect();
    for k in before.keys().filter(|k| !live.contains_key(k)) {
        c.insert(*k, t);
    }
    c.retain(|k, at| live.contains_key(k) || *at > t - KEEP);
}

/// Junta duas versões da mesma nota: cada campo, item e imagem fica com o carimbo mais
/// novo. Dá o mesmo resultado nos dois sentidos, então os aparelhos convergem. Se a
/// junção quebrar a regra do tipo (51 imagens, por exemplo), vale a versão mais nova inteira.
pub fn merge(a: &Note, b: &Note) -> Note {
    let (new, old) = if newer(a, b) { (a, b) } else { (b, a) };
    if a.kind != b.kind {
        return new.clone();
    }
    let mut n = new.clone();
    let (c, o) = (&mut n.clock, &old.clock);
    pick(&mut n.title, &mut c.title, &old.title, o.title);
    pick(&mut n.body, &mut c.body, &old.body, o.body);
    pick(&mut n.pinned, &mut c.pinned, &old.pinned, o.pinned);
    pick(&mut n.rank, &mut c.rank, &old.rank, o.rank);
    (n.items, c.items) = join(&new.items, &new.clock.items, &old.items, &o.items, |i| i.id);
    (n.images, c.images) = join(&new.images, &new.clock.images, &old.images, &o.images, |i| i.id);
    n.created = a.created.min(b.created);
    n.updated = a.updated.max(b.updated);
    if n.check().is_err() { new.clone() } else { n }
}

/// Qual das duas é a mais nova. No empate decide o conteúdo, para os dois lados escolherem igual.
fn newer(a: &Note, b: &Note) -> bool {
    match a.updated.cmp(&b.updated) {
        std::cmp::Ordering::Equal => {
            let s = |n: &Note| serde_json::to_string(&(n, &n.clock)).unwrap_or_default();
            s(a) >= s(b)
        }
        o => o.is_gt(),
    }
}

fn pick<T: Clone>(v: &mut T, at: &mut i64, other: &T, other_at: i64) {
    if other_at > *at {
        *v = other.clone();
        *at = other_at;
    }
}

/// Itens (ou imagens) dos dois lados: cada um fica com o lado de carimbo mais novo, que
/// pode ser a remoção. A ordem é a da versão mais nova, e o que só a outra tem vai no fim.
fn join<T: Clone>(
    new: &[T],
    nc: &BTreeMap<Id, i64>,
    old: &[T],
    oc: &BTreeMap<Id, i64>,
    id: impl Fn(&T) -> Id,
) -> (Vec<T>, BTreeMap<Id, i64>) {
    let at = |c: &BTreeMap<Id, i64>, k: &Id| c.get(k).copied().unwrap_or(0);
    let theirs = |k: &Id| at(oc, k) > at(nc, k);
    let mut out = Vec::with_capacity(new.len().max(old.len()));
    for x in new {
        let k = id(x);
        if !theirs(&k) {
            out.push(x.clone());
        } else if let Some(y) = old.iter().find(|y| id(y) == k) {
            out.push(y.clone());
        }
    }
    for y in old {
        let k = id(y);
        if theirs(&k) && !new.iter().any(|x| id(x) == k) {
            out.push(y.clone());
        }
    }
    let mut c = nc.clone();
    for (k, t) in oc {
        let e = c.entry(*k).or_insert(*t);
        *e = (*e).max(*t);
    }
    (out, c)
}
