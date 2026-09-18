CREATE TABLE IF NOT EXISTS keys
(
    service TEXT NOT NULL PRIMARY KEY,
    key VARCHAR(32) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO keys (service, key, is_active)
VALUES
(
 'service-a',
'qwertyuiopfdasdfghjklelzxcvbnmfd',
 true,
);