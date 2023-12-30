use unicode_segmentation::UnicodeSegmentation;

static FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '\\', '"', '<', '>', '{', '}'];

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let too_long = s.graphemes(true).count() > 256;
        let has_illegal_characters = s.chars().any(|g| FORBIDDEN_CHARACTERS.contains(&g));

        if too_long || is_empty_or_whitespace || has_illegal_characters {
            Err(format!("{s} is not a valid subscriber name").to_string())
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "ż".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_257_grapheme_long_name_is_invalid() {
        let name = "ż".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Arsene Lupin".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
