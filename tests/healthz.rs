use std::net::TcpListener;

fn spawn_app() -> String {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Could not bind address");

    tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn test() {
    // GIVEN
    let address = spawn_app();

    let client = reqwest::Client::new();

    // WHEN
    let result = client
        .get(format!("{address}/healthz"))
        .send()
        .await
        .expect("Failed to send request");

    // THEN
    assert!(result.status().is_success());
    assert_eq!(result.content_length(), Some(0));
}

#[tokio::test]
async fn subscription_returns_200_on_correct_body() {
    // GIVEN
    let address = spawn_app();
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // WHEN
    let result = client
        .post(format!("{address}/subscribe"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Could not send request");

    // THEN

    assert_eq!(200, result.status());
}

#[tokio::test]
async fn subscription_returns_400_on_malformed_body() {
    // GIVEN
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = [
        ("name=lupin", "no email"),
        ("email=arsene@lup.in", "no name"),
        ("", "no name or email"),
    ];

    for (body, message) in test_cases {
        // WHEN
        let result = client
            .post(format!("{address}/subscribe"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Could not send request");
        // THEN

        assert_eq!(
            400,
            result.status(),
            "The API did not fail properly with Bad Request (400) when the body had {message}"
        );
    }
}
