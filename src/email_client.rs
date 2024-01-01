use std::time::Duration;

use crate::domain::Email;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

#[derive(Clone)]
pub struct EmailClient {
    base_url: String,
    http_client: reqwest::Client,
    sender: Email,
    token: Secret<String>,
    timeout: Duration,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequestBody<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    text_body: &'a str,
    html_body: &'a str,
}

/* #[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EmailSubmissionResult {
    error_code: i64,
    message: String,
    #[serde(rename = "MessageID")]
    message_id: Uuid,
    submitted_at: DateTime<chrono::Utc>,
} */

impl EmailClient {
    pub fn new(base_url: String, sender: Email, token: Secret<String>, timeout: Duration) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url,
            sender,
            token,
            timeout,
        }
    }

    pub async fn send_email(
        &self,
        recipient: Email,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let body = SendEmailRequestBody {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            text_body,
            subject,
            html_body,
        };

        let _ = self
            .http_client
            .post(url)
            .timeout(self.timeout)
            .json(&body)
            .header("X-Postmark-Server-Token", self.token.expose_secret())
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use claims::{assert_err, assert_ok};
    use fake::{
        faker::{internet::en::SafeEmail, lorem::en::Paragraph, lorem::en::Sentence},
        Fake,
    };
    use wiremock::{
        matchers::{any, header, header_exists, method},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::domain::Email;

    use super::EmailClient;

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, serde_json::Error> =
                serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    fn email() -> Email {
        Email::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            secrecy::Secret::new(fake::Faker.fake()),
            Duration::from_millis(200),
        )
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_times_out() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500).set_delay(Duration::from_secs(30)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(result);
    }
}
