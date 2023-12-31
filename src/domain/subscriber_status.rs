#[derive(sqlx::Type)]
#[sqlx(type_name = "subscriber_status", rename_all = "snake_case")]
pub enum SubscriberStatus {
    PendingConfirmation,
    Ok,
}
