use crate::helpers::spawn_app;

use crate::helpers::{email, name};
use urlencoding::encode;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};
use zero2prod::domain::SubscriberStatus;
use zero2prod::routes::generate_subscription_token;

fn build_body(name: &str, email: &str) -> String {
    format!("name={}&email={}", encode(name), encode(email))
}

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_400() {
    // GIVEN
    let app = spawn_app().await;

    // WHEN
    let response = reqwest::get(&format!("{}/subscribe/confirm", app.connection_string))
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn confirmations_with_unpersisted_token_are_rejected_with_401() {
    // GIVEN
    let app = spawn_app().await;

    // WHEN
    let response = reqwest::get(&format!(
        "{}/subscribe/confirm?token={}",
        app.connection_string,
        generate_subscription_token()
    ))
    .await
    .unwrap();

    // THEN
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn link_returns_200_on_correct_token() {
    // GIVEN
    let app = spawn_app().await;
    let name = name();
    let email = email();
    let body = build_body(&name, &email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body).await;
    let request = &app.email_server.received_requests().await.unwrap()[0];

    let links = app.get_confirmation_links(request);

    // WHEN
    let response = reqwest::get(links.html).await.unwrap();

    // THEN
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn after_confirmation_account_is_no_longer_pending() {
    // GIVEN
    let app = spawn_app().await;
    let name = name();
    let email = email();
    let body = build_body(&name, &email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body).await;
    let request = &app.email_server.received_requests().await.unwrap()[0];

    let links = app.get_confirmation_links(request);

    // WHEN
    reqwest::get(links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!(
        "SELECT status FROM subscriptions WHERE name = $1 AND email = $2",
        name,
        email
    )
    .fetch_one(&app.database)
    .await
    .expect("Failed to get subscriptions");

    // THEN
    assert_eq!(saved.status, SubscriberStatus::Ok.to_string());
}

#[tokio::test]
async fn trying_to_confirm_a_confirmed_account_returns_400() {
    // GIVEN
    let app = spawn_app().await;
    let name = name();
    let email = email();
    let body = build_body(&name, &email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body).await;
    let request = &app.email_server.received_requests().await.unwrap()[0];

    let links = app.get_confirmation_links(request);

    // WHEN
    reqwest::get(links.html.clone())
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let result = reqwest::get(links.html).await.unwrap();

    // THEN
    assert_eq!(result.status(), 400);
}
