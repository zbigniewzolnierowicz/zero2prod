#[derive(sqlx::Type, Debug, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum SubscriberStatus {
    PendingConfirmation,
    Ok,
}
