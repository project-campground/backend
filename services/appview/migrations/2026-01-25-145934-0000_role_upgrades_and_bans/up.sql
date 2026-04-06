CREATE TABLE IF NOT EXISTS appview.campsite_ban (
    campsiteId character varying NOT NULL,
    userId character varying NOT NULL,
    reason character varying,
    createdBy character varying NOT NULL,
    createdAt timestamp NOT NULL,
    updatedBy character varying NOT NULL,
    updatedAt timestamp NOT NULL,
    PRIMARY KEY(campsiteId, userId)
);

ALTER TABLE appview.campsite_role
ADD COLUMN IF NOT EXISTS members text array NOT NULL
DEFAULT '{}';

ALTER TABLE appview.campsite_role
ADD COLUMN IF NOT EXISTS flags integer NOT NULL
DEFAULT 1;

ALTER TABLE appview.campsite_invite
ADD COLUMN IF NOT EXISTS used integer NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_invite
ALTER COLUMN id TYPE uuid USING gen_random_uuid();

CREATE INDEX IF NOT EXISTS campsite_invite_id_idx
    ON appview.campsite_invite (id);

WITH r AS (SELECT userid, unnest(roles) as roleid FROM appview.campsite_member),
     rm AS (SELECT array_agg(userid) as userids, roleid FROM r GROUP BY roleid)
UPDATE appview.campsite_role SET members = (SELECT userids FROM rm WHERE rm.roleid = appview.campsite_role.id);