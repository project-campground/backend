-- Permissions
ALTER TABLE appview.campsite_permission RENAME COLUMN allowedGeneralPermissions TO allowedCampsitePermissions;
ALTER TABLE appview.campsite_permission RENAME COLUMN deniedGeneralPermissions TO deniedCampsitePermissions;

ALTER TABLE appview.campsite_permission RENAME COLUMN allowedContentPermissions TO allowedTentPermissions;
ALTER TABLE appview.campsite_permission RENAME COLUMN deniedContentPermissions TO deniedTentPermissions;
-- Roles
ALTER TABLE appview.campsite_role RENAME COLUMN generalPermissions TO campsitePermissions;
ALTER TABLE appview.campsite_role RENAME COLUMN contentPermissions TO tentPermissions;