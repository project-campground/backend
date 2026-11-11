-- Create profile post table
CREATE TABLE IF NOT EXISTS appview.profile_post (
    uri character varying PRIMARY KEY,
    cid character varying NOT NULL,
    author character varying NOT NULL,
    parentUri character varying,
    content character varying NOT NULL,
    tags text array NOT NULL,
    replies text array NOT NULL,
    indexedAt character varying NOT NULL,
    createdAt character varying NOT NULL,
    updatedAt character varying
);

CREATE INDEX profile_post_uri_idx
    ON appview.profile_post (uri);