-- Discord-like servers
CREATE TABLE IF NOT EXISTS appview.campsite (
    id character varying PRIMARY KEY,
    name character varying NOT NULL,
    vanityUrl character varying,
    description character varying NOT NULL,
    avatarUri character varying,
    bannerUri character varying,
    tags text array NOT NULL,
    -- For faster fetching
    memberDids text array NOT NULL,
    owner character varying NOT NULL,
    createdBy character varying NOT NULL,
    createdAt timestamp NOT NULL,
    updatedBy character varying NOT NULL,
    updatedAt timestamp NOT NULL
);
CREATE TABLE IF NOT EXISTS appview.campsite_invite (
    id character varying PRIMARY KEY,
    campsiteId character varying NOT NULL,
    allowedAmount integer,
    expiresAt timestamp,
    -- For faster fetching
    createdBy character varying NOT NULL,
    createdAt timestamp NOT NULL
);
-- Guilded-like groups
CREATE TABLE IF NOT EXISTS appview.bonfire (
    id character varying PRIMARY KEY,
    campsiteId character varying NOT NULL,
    name character varying NOT NULL,
    description character varying NOT NULL,
    avatarUri character varying,
    bannerUri character varying,
    priority integer NOT NULL,
    createdBy character varying NOT NULL,
    createdAt timestamp NOT NULL,
    updatedBy character varying NOT NULL,
    updatedAt timestamp NOT NULL
);
-- Discord-like categories
CREATE TABLE IF NOT EXISTS appview.tent_category (
    id uuid PRIMARY KEY,
    campsiteId character varying NOT NULL,
    bonfireId character varying NOT NULL,
    name character varying NOT NULL,
    description character varying NOT NULL,
    priority integer NOT NULL,
    createdBy character varying NOT NULL,
    createdAt timestamp NOT NULL,
    updatedBy character varying NOT NULL,
    updatedAt timestamp NOT NULL
);
-- Discord-like channels
CREATE TABLE IF NOT EXISTS appview.tent (
    id uuid PRIMARY KEY,
    campsiteId character varying NOT NULL,
    bonfireId character varying NOT NULL,
    categoryId uuid,
    name character varying NOT NULL,
    type integer NOT NULL,
    viewType integer NOT NULL,
    description character varying NOT NULL,
    priority integer NOT NULL,
    createdBy character varying NOT NULL,
    createdAt timestamp NOT NULL,
    updatedBy character varying NOT NULL,
    updatedAt timestamp NOT NULL
);
-- Messages
CREATE TABLE IF NOT EXISTS appview.tent_message (
    id uuid PRIMARY KEY,
    campsiteId character varying NOT NULL,
    tentId uuid NOT NULL,
    content character varying NOT NULL,
    replyingTo uuid array NOT NULL,
    createdBy character varying NOT NULL,
    createdAt timestamp NOT NULL,
    updatedAt timestamp
);
CREATE TABLE IF NOT EXISTS appview.campsite_member (
    userId character varying NOT NULL,
    campsiteId character varying NOT NULL,
    joinedAt timestamp NOT NULL,
    usedInviteId character varying,
    nickname character varying,
    PRIMARY KEY (userId, campsiteId)
);

-- The servers user is in
ALTER TABLE appview.actor
ADD COLUMN IF NOT EXISTS campsites text array NOT NULL
DEFAULT '{}';

CREATE INDEX campsite_id_idx
    ON appview.campsite (id);
CREATE INDEX bonfire_id_idx
    ON appview.bonfire (id);
CREATE INDEX tent_category_id_idx
    ON appview.tent_category (id);
CREATE INDEX tent_id_idx
    ON appview.tent (id);
CREATE INDEX tent_message_id_idx
    ON appview.tent_message (id);