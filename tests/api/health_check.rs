use crate::helpers::spawn_app;

#[tokio::test]
async fn test() {
    // GIVEN
    let app = spawn_app().await;

    // WHEN
    let result = app.healthcheck().await;

    // THEN
    assert!(result.status().is_success());
    assert_eq!(result.content_length(), Some(0));
}
