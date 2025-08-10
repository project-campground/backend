-- Create Block table
CREATE TABLE IF NOT EXISTS appview.block (
    id bigserial PRIMARY KEY,
    did character varying NOT NULL,
    targetDid character varying NOT NULL,
    UNIQUE (did, targetDid)
);

-- Create Friend table
CREATE TABLE IF NOT EXISTS appview.friend (
    id bigserial PRIMARY KEY,
    did character varying NOT NULL,
    targetDid character varying NOT NULL,
    UNIQUE (did, targetDid)
);

-- Create Account table
CREATE TABLE IF NOT EXISTS appview.account (
    did character varying PRIMARY KEY,
    settings jsonb NOT NULL DEFAULT '{}'::jsonb,
    socialConnections jsonb NOT NULL DEFAULT '{}'::jsonb,
    activities jsonb NOT NULL DEFAULT '{}'::jsonb,
    status character varying NOT NULL DEFAULT 'online',
    statusText character varying,
    statusEmoji character varying,
    lastSeen timestamp NOT NULL DEFAULT NOW(),
    email character varying
);

-- event sequencer implementation
-- Create Event Sequence Table
CREATE TABLE IF NOT EXISTS appview.event_seq (
    seq bigserial PRIMARY KEY,
    homeServer character varying NOT NULL,
    eventType character varying NOT NULL,
    event jsonb NOT NULL,
    invalidated smallint NOT NULL DEFAULT 0,
    sequencedAt timestamp NOT NULL
);
CREATE INDEX event_seq_home_server_idx
    ON appview.event_seq (homeServer);
CREATE INDEX event_seq_event_type_idx
    ON appview.event_seq (eventType);
CREATE INDEX event_seq_sequenced_at_idx
    ON appview.event_seq (sequencedAt);