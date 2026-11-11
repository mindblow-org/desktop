use serde_json::Value;

pub fn parse(body: &str) -> Option<Vec<Value>> {
    let v: Value = serde_json::from_str(body).ok()?;
    v.get("els")?.as_array().cloned()
}

pub fn words(body: &str) -> String {
    let els = parse(body).unwrap_or_default();
    let texts = els.iter().filter(|e| e["t"] == "text").filter_map(|e| e["s"].as_str());
    texts.collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::{parse, words};

    #[test]
    fn reads_only_the_texts() {
        let b = r#"{"v":1,"els":[{"t":"text","s":"Fluxo"},{"t":"rect","x":0},{"t":"text","s":"login"}]}"#;
        assert_eq!(words(b), "Fluxo\nlogin");
    }

    #[test]
    fn needs_an_element_list() {
        assert!(parse(r#"{"v":1,"els":[]}"#).is_some());
        assert!(parse(r#"{"v":1}"#).is_none());
        assert!(parse("rabisco").is_none());
    }
}
