use actix_web::HttpResponse;

pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[cfg(test)]
mod tests {
    use super::ping;

    #[tokio::test]
    async fn ping_works() {
        // GIVEN, WHEN
        let result = ping().await;

        // THEN
        assert!(result.status().is_success());
    }
}
