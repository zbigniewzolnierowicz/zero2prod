use crate::helpers::spawn_app;
use fake::{
    faker::{internet::en::SafeEmail, name::en::FirstName},
    Fake,
};
use urlencoding::encode;
use wiremock::{Mock, matchers::{path, method}, ResponseTemplate};

#[tokio::test]
async fn subscription_returns_200_on_correct_body() {
    // GIVEN
    let app = spawn_app().await;
    let name: String = FirstName().fake();
    let email: String = SafeEmail().fake();
    let body = format!("name={}&email={}", encode(&name), encode(&email));

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // WHEN
    let result = app.post_subscriptions(body.to_string()).await;

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.database)
        .await
        .expect("Failed to get subscriptions");

    // THEN
    assert_eq!(200, result.status());
    assert_eq!(saved.name, name);
    assert_eq!(saved.email, email);
}

#[tokio::test]
async fn subscription_returns_400_on_malformed_body() {
    // GIVEN
    let app = spawn_app().await;
    let test_cases = [
        ("name=lupin", "no email"),
        ("email=arsene@lup.in", "no name"),
        ("", "no name or email"),
        ("name=&email=arsene@lup.in", "empty name"),
        ("name=lupin&email=", "empty email"),
        ("name=&email=", "empty name and email"),
        ("name=Lupin&email=invalidemail.com", "no @ sign in email"),
        ("name=Lupin&email=thing@", "no domain in email"),
        ("name=Lupin&email=@domain.com", "no user in email"),
    ];

    for (invalid_body, why_invalid_body_message) in test_cases {
        // WHEN
        let result = app.post_subscriptions(invalid_body.to_string()).await;

        // THEN
        assert_eq!(
            400,
            result.status(),
            "The API did not fail properly with Bad Request (400) when the body had {why_invalid_body_message}"
        );
    }
}

#[tokio::test]
async fn subscription_sends_a_confirmation_email_for_valid_data() {
    // GIVEN
    let app = spawn_app().await;
    let name: String = FirstName().fake();
    let email: String = SafeEmail().fake();
    let body = format!("name={}&email={}", encode(&name), encode(&email));

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // WHEN
    let result = app.post_subscriptions(body.to_string()).await;

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.database)
        .await
        .expect("Failed to get subscriptions");

    // THEN
    assert_eq!(200, result.status());
    assert_eq!(saved.name, name);
    assert_eq!(saved.email, email);
}
