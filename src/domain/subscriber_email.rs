#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty = s.trim().is_empty();
        let valid_email = validator::validate_email(s.clone());

        if is_empty || !valid_email {
            Err(format!("{s} is not a valid email"))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Email;
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_email_passes_without_issue(valid_email: ValidEmailFixture) -> bool {
        Email::parse(valid_email.0).is_ok()
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(Email::parse(email));
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(Email::parse(email));
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(Email::parse(email));
    }
}
