CREATE TABLE IF NOT EXISTS appview.campsite_role (
    id uuid PRIMARY KEY,
    campsiteId character varying NOT NULL,
    name character varying NOT NULL,
    displaySeparately boolean NOT NULL,
    mentionable boolean NOT NULL,
    campsitePermissions bigint NOT NULL,
    tentPermissions bigint NOT NULL,
    color integer NOT NULL,
    colorSecondary integer NOT NULL,
    priority integer NOT NULL,
    createdBy character varying NOT NULL,
    createdAt timestamp NOT NULL,
    updatedBy character varying NOT NULL,
    updatedAt timestamp NOT NULL
);
-- Might need to split this table into bonfire, category and tent permissions
CREATE TABLE IF NOT EXISTS appview.campsite_permission (
    id uuid PRIMARY KEY,
    campsiteId character varying NOT NULL,
    -- For who
    roleId uuid,
    userId character varying,
    -- Specify where the permission is applied
    bonfireId character varying,
    categoryId uuid,
    tentId uuid,

    allowedCampsitePermissions bigint NOT NULL,
    deniedCampsitePermissions bigint NOT NULL,
    allowedTentPermissions bigint NOT NULL,
    deniedTentPermissions bigint NOT NULL,

    createdBy character varying NOT NULL,
    createdAt timestamp NOT NULL,
    updatedBy character varying NOT NULL,
    updatedAt timestamp NOT NULL
);

ALTER TABLE appview.campsite_member
ADD COLUMN IF NOT EXISTS roles uuid array NOT NULL
DEFAULT '{}';