CREATE TABLE subscription_tokens (
    subscription_token TEXT NOT NULL,
    subscriber_id UUID NOT NULL,

    PRIMARY KEY (subscription_token),
    CONSTRAINT fk_subscriber_id FOREIGN KEY (
        subscriber_id
    ) REFERENCES subscriptions (id)
)
