-- Create AppView schema
CREATE SCHEMA IF NOT EXISTS appview;

-- Create Profile table
CREATE TABLE IF NOT EXISTS appview.profile (
    uri character varying PRIMARY KEY,
    cid character varying NOT NULL,
    creator character varying NOT NULL,
    displayName character varying,
    description character varying,
    avatarCid character varying,
    bannerCid character varying,
    indexedAt character varying NOT NULL
);

CREATE INDEX profile_creator_idx
    ON appview.profile (creator);

-- Create Actor table
CREATE TABLE IF NOT EXISTS appview.actor (
    did character varying PRIMARY KEY,
    handle character varying,
    indexedAt character varying NOT NULL,
    CONSTRAINT actor_handle UNIQUE (handle)
);
