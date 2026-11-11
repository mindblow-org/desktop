pub fn between(a: Option<f64>, b: Option<f64>) -> f64 {
    match (a, b) {
        (None, None) => 0.0,
        (Some(a), None) => a + 1.0,
        (None, Some(b)) => b - 1.0,
        (Some(a), Some(b)) => (a + b) / 2.0,
    }
}

#[cfg(test)]
mod tests {
    use super::between;

    #[test]
    fn first_note_starts_at_zero() {
        assert_eq!(between(None, None), 0.0);
    }

    #[test]
    fn edges_step_by_one() {
        assert_eq!(between(None, Some(3.0)), 2.0);
        assert_eq!(between(Some(3.0), None), 4.0);
    }

    #[test]
    fn middle_splits_gap() {
        let r = between(Some(1.0), Some(2.0));
        assert!(r > 1.0 && r < 2.0);
    }
}
