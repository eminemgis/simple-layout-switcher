use std::collections::HashMap;

/// Bidirectional character map between two keyboard layouts.
#[derive(Clone)]
pub struct LayoutPair {
    pub id: &'static str,
    pub label: &'static str,
    forward: HashMap<char, char>,
    backward: HashMap<char, char>,
}

impl LayoutPair {
    fn from_rows(id: &'static str, label: &'static str, a: &str, b: &str) -> Self {
        assert_eq!(a.chars().count(), b.chars().count());
        let mut forward = HashMap::new();
        let mut backward = HashMap::new();
        for (ca, cb) in a.chars().zip(b.chars()) {
            forward.insert(ca, cb);
            backward.insert(cb, ca);
        }
        Self {
            id,
            label,
            forward,
            backward,
        }
    }

    pub fn convert(&self, text: &str) -> String {
        text.chars()
            .map(|c| {
                self.forward
                    .get(&c)
                    .or_else(|| self.backward.get(&c))
                    .copied()
                    .unwrap_or(c)
            })
            .collect()
    }
}

pub fn en_ru() -> LayoutPair {
    // Physical key pairs: US QWERTY ↔ Russian ЙЦУКЕН
    let en = "`~qwertyuiop[]asdfghjkl;'zxcvbnm,./QWERTYUIOP{}ASDFGHJKL:\"ZXCVBNM<>?";
    let ru = "ёЁйцукенгшщзхъфывапролджэячсмитьбю.ЙЦУКЕНГШЩЗХЪФЫВАПРОЛДЖЭЯЧСМИТЬБЮ,";
    LayoutPair::from_rows("en-ru", "English ↔ Russian", en, ru)
}

pub fn available_pairs() -> Vec<LayoutPair> {
    vec![en_ru()]
}

pub fn pair_by_id(id: &str) -> Option<LayoutPair> {
    available_pairs().into_iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_word_both_ways() {
        let pair = en_ru();
        assert_eq!(pair.convert("ghbdtn"), "привет");
        assert_eq!(pair.convert("привет"), "ghbdtn");
        assert_eq!(pair.convert("Ghbdtn"), "Привет");
    }

    #[test]
    fn leaves_unknown_chars() {
        let pair = en_ru();
        assert_eq!(pair.convert("hi 123!"), "рш 123!");
    }
}
