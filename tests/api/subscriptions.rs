use crate::helpers::spawn_app;

use crate::helpers::{email, name};
use urlencoding::encode;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};
use zero2prod::domain::SubscriberStatus;

fn build_body(name: &str, email: &str) -> String {
    format!("name={}&email={}", encode(name), encode(email))
}

#[tokio::test]
async fn subscription_returns_200_on_correct_body() {
    // GIVEN
    let app = spawn_app().await;
    let name: String = name();
    let email: String = email();
    let body = build_body(&name, &email);

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
async fn subscription_persists_new_subscriber() {
    // GIVEN
    let app = spawn_app().await;
    let name: String = name();
    let email: String = email();
    let body = build_body(&name, &email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // WHEN
    app.post_subscriptions(body.to_string()).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.database)
        .await
        .expect("Failed to get subscriptions");

    // THEN
    assert_eq!(saved.name, name);
    assert_eq!(saved.email, email);
    assert_eq!(
        saved.status,
        SubscriberStatus::PendingConfirmation.to_string()
    );
}

#[tokio::test]
async fn subscription_sends_a_confirmation_email_for_valid_data() {
    // GIVEN
    let app = spawn_app().await;
    let name: String = name();
    let email: String = email();
    let body = build_body(&name, &email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // WHEN
    app.post_subscriptions(body.to_string()).await;

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.database)
        .await
        .expect("Failed to get subscriptions");

    // THEN
    assert_eq!(saved.name, name);
    assert_eq!(saved.email, email);
}

#[tokio::test]
async fn subscription_sends_a_confirmation_email_with_link() {
    // GIVEN
    let app = spawn_app().await;
    let name: String = name();
    let email: String = email();
    let body = build_body(&name, &email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // WHEN
    app.post_subscriptions(body).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let links = app.get_confirmation_links(email_request);

    // THEN
    assert_eq!(links.html, links.plain_text);
}

#[tokio::test]
async fn subscribing_twice_sends_the_same_link() {
    // GIVEN
    let app = spawn_app().await;
    let name: String = name();
    let email: String = email();
    let body = build_body(&name, &email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    // WHEN
    app.post_subscriptions(body.clone())
        .await
        .error_for_status()
        .unwrap();
    app.post_subscriptions(body)
        .await
        .error_for_status()
        .unwrap();

    let requests = &app.email_server.received_requests().await.unwrap();
    let links: Vec<_> = requests
        .iter()
        .map(|r| app.get_confirmation_links(r).html)
        .collect();

    // THEN

    assert!(links.windows(2).all(|a| a[0] == a[1]))
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // Sabotage the database
    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&app.database)
        .await
        .unwrap();
    // Act
    let response = app.post_subscriptions(body.into()).await;
    // Assert
    assert_eq!(response.status().as_u16(), 500);
}
