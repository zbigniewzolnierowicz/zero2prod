CREATE TYPE subscriber_status AS ENUM ('ok', 'pending_confirmation');
ALTER TABLE subscriptions ADD status subscriber_status NOT NULL DEFAULT 'pending_confirmation';
