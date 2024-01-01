use zero2prod::configuration::Settings;
use zero2prod::startup::Application;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = Settings::get().expect("Failed to read configuration.");
    let app = Application::build(config).await?;
    app.server.await?;

    Ok(())
}
