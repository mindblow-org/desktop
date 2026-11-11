use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

use crate::{Kind, Note, note::scene};

pub fn fold(s: &str) -> String {
    s.nfd().filter(|c| !is_combining_mark(*c)).flat_map(char::to_lowercase).collect()
}

pub fn hit(n: &Note, q: &str) -> bool {
    let hay = fold(&text(n));
    fold(q).split_whitespace().all(|t| hay.contains(t))
}

fn text(n: &Note) -> String {
    let body = if n.kind == Kind::Board { scene::words(&n.body) } else { n.body.clone() };
    let mut s = format!("{}\n{}", n.title, body);
    for i in &n.items {
        s.push('\n');
        s.push_str(&i.text);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::fold;

    #[test]
    fn folds_case_and_accents() {
        assert_eq!(fold("Pão de Açúcar"), "pao de acucar");
        assert_eq!(fold("REUNIÃO"), "reuniao");
    }
}
