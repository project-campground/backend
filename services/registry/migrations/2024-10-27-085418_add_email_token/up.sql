-- Your SQL goes here
-- Create Email Token Table
CREATE TABLE IF NOT EXISTS registry.email_token (
    purpose character varying NOT NULL,
    did character varying NOT NULL,
    token character varying NOT NULL,
    "requestedAt" character varying NOT NULL
);
ALTER TABLE ONLY registry.email_token
    DROP CONSTRAINT IF EXISTS email_token_pkey;
ALTER TABLE ONLY registry.email_token
    ADD CONSTRAINT email_token_pkey PRIMARY KEY (purpose, did);
CREATE UNIQUE INDEX email_token_purpose_token_unique
	ON registry.email_token (purpose, token);