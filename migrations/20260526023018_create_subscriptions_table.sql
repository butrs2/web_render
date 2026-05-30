CREATE TABLE subscriptions
(
    id            UUID PRIMARY KEY,
    email         TEXT        NOT NULL UNIQUE,
    name          TEXT        NOT NULL,
    subscribed_at TIMESTAMPTZ NOT NULL
);

-- 增加一个状态字段，默认为 'pending_confirmation'
ALTER TABLE subscriptions ADD COLUMN status TEXT NOT NULL DEFAULT 'pending_confirmation';