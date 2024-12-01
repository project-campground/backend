-- This file should undo anything in `up.sql`
DROP TABLE registry.repo_seq;
DROP TABLE registry.did_doc;
DROP TABLE registry.account_pref;
DROP TABLE registry.backlink;
DROP TABLE registry.record_blob;
DROP TABLE registry.blob;
DROP TABLE registry.record;
DROP TABLE registry.repo_block;
DROP TABLE registry.repo_root;
DROP TABLE registry.account;
DROP TABLE registry.actor;
DROP TABLE registry.refresh_token;
DROP TABLE registry.app_password;
DROP SCHEMA IF EXISTS registry;