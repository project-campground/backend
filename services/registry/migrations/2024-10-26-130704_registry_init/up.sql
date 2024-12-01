-- Create Registry schema
CREATE SCHEMA IF NOT EXISTS registry;

/* Based heavily on account-manager, did-cache, sequencer, and actor-store migrations
    from the canonical TS implementation. */

-- account-manager implementation
-- Create App Password Table
CREATE TABLE IF NOT EXISTS registry.app_password (
    did character varying NOT NULL,
    name character varying NOT NULL,
    "password" character varying NOT NULL,
    "createdAt" character varying NOT NULL
);

ALTER TABLE ONLY registry.app_password
    DROP CONSTRAINT IF EXISTS app_password_pkey;
ALTER TABLE ONLY registry.app_password
    ADD CONSTRAINT app_password_pkey PRIMARY KEY (did, name);

-- Create Refresh Token Table
CREATE TABLE IF NOT EXISTS registry.refresh_token (
    id character varying PRIMARY KEY,
    did character varying NOT NULL,
    "expiresAt" character varying NOT NULL,
    "nextId" character varying,
    "appPasswordName" character varying
);
CREATE INDEX refresh_token_did_idx -- Aids in refresh token cleanup
    ON registry.refresh_token(did);

-- Create Actor Table
CREATE TABLE IF NOT EXISTS registry.actor (
    did character varying PRIMARY KEY,
    handle character varying,
    "createdAt" character varying NOT NULL,
    "deactivatedAt" character varying,
    "deleteAfter" character varying,
    "takedownRef" character varying
);
CREATE UNIQUE INDEX actor_handle_lower_idx
    ON registry.actor (LOWER(handle));
CREATE INDEX actor_cursor_idx
    ON registry.actor("createdAt", did);

-- Create Account Table
CREATE TABLE IF NOT EXISTS registry.account (
    did character varying PRIMARY KEY,
    email character varying NOT NULL,
    "recoveryKey" character varying, -- For storing Bring Your Own Key
    "password" character varying NOT NULL,
    "createdAt" character varying NOT NULL,
	"emailConfirmedAt" character varying
);
CREATE UNIQUE INDEX account_email_lower_idx
	ON registry.account (LOWER(email));
CREATE INDEX account_cursor_idx
	ON registry.account("createdAt", did);


-- actor-store implementation
-- Create Repo Root Table
CREATE TABLE IF NOT EXISTS registry.repo_root (
    did character varying PRIMARY KEY,
    cid character varying NOT NULL,
    rev character varying NOT NULL,
    "indexedAt" character varying NOT NULL
);

-- Create Repo Block Table
CREATE TABLE IF NOT EXISTS registry.repo_block (
    cid character varying NOT NULL,
    did character varying NOT NULL,
    "repoRev" character varying NOT NULL,
    size integer NOT NULL,
    content bytea NOT NULL
);
ALTER TABLE ONLY registry.repo_block
    ADD CONSTRAINT repo_block_pkey PRIMARY KEY (cid, did);
CREATE INDEX repo_block_repo_rev_idx
	ON registry.repo_block("repoRev", cid);

-- Create Record Table
CREATE TABLE IF NOT EXISTS registry.record (
    uri character varying PRIMARY KEY,
    cid character varying NOT NULL,
    did character varying NOT NULL,
    collection character varying NOT NULL,
    "rkey" character varying NOT NULL,
    "repoRev" character varying,
    "indexedAt" character varying NOT NULL,
    "takedownRef" character varying
);
CREATE INDEX record_did_cid_idx
	ON registry.record(cid);
CREATE INDEX record_did_collection_idx
	ON registry.record(collection);
CREATE INDEX record_repo_rev_idx
	ON registry.record("repoRev");

-- Create Blob Table
CREATE TABLE IF NOT EXISTS registry.blob (
    cid character varying NOT NULL,
    did character varying NOT NULL,
    "mimeType" character varying NOT NULL,
    size integer NOT NULL,
    "tempKey" character varying,
    width integer,
    height integer,
    "createdAt" character varying NOT NULL,
    "takedownRef" character varying
);
ALTER TABLE ONLY registry.blob
    ADD CONSTRAINT blob_pkey PRIMARY KEY (cid, did);
CREATE INDEX blob_tempkey_idx
	ON registry.blob("tempKey");

-- Create Record Blob Table
CREATE TABLE IF NOT EXISTS registry.record_blob (
    "blobCid" character varying NOT NULL,
    "recordUri" character varying NOT NULL,
    did character varying NOT NULL
);
ALTER TABLE ONLY registry.record_blob
    DROP CONSTRAINT IF EXISTS record_blob_pkey;
ALTER TABLE ONLY registry.record_blob
    ADD CONSTRAINT record_blob_pkey PRIMARY KEY ("blobCid","recordUri");

-- Create Backlink Table
CREATE TABLE IF NOT EXISTS registry.backlink (
    uri character varying NOT NULL,
    path character varying NOT NULL,
    "linkTo" character varying NOT NULL
);
ALTER TABLE ONLY registry.backlink
    DROP CONSTRAINT IF EXISTS backlink_pkey;
ALTER TABLE ONLY registry.backlink
    ADD CONSTRAINT backlink_pkey PRIMARY KEY (uri, path);
CREATE INDEX backlink_link_to_idx
	ON registry.backlink(path, "linkTo");

-- Create Account Preferences Table
CREATE TABLE IF NOT EXISTS registry.account_pref (
	id SERIAL PRIMARY KEY,
    did character varying NOT NULL,
    name character varying NOT NULL,
    "valueJson" text
);

-- did-cache implementation
-- Create DID Cache Table
CREATE TABLE IF NOT EXISTS registry.did_doc (
    did character varying PRIMARY KEY,
    doc text NOT NULL,
    "updatedAt" bigint NOT NULL
);

-- sequencer implementation
-- Create Repo Sequence Table
CREATE TABLE IF NOT EXISTS registry.repo_seq (
    seq bigserial PRIMARY KEY,
    did character varying NOT NULL,
    "eventType" character varying NOT NULL,
    event bytea NOT NULL,
    invalidated smallint NOT NULL DEFAULT 0,
    "sequencedAt" character varying NOT NULL
);
CREATE INDEX repo_seq_did_idx -- for filtering seqs based on did
	ON registry.repo_seq(did);
CREATE INDEX repo_seq_event_type_idx -- for filtering seqs based on event type
	ON registry.repo_seq("eventType");
CREATE INDEX repo_seq_sequenced_at_index -- for entering into the seq stream at a particular time
	ON registry.repo_seq("sequencedAt");