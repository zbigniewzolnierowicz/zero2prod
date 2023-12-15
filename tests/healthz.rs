use std::net::TcpListener;

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

fn spawn_app() -> String {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Could not bind address");

    tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
