DEFINE TABLE SCHEMAFULL repo_root;
DEFINE FIELD cid ON TABLE repo_root TYPE string;
DEFINE FIELD rev ON TABLE repo_root TYPE string;
DEFINE FIELD indexedAt ON TABLE repo_root TYPE datetime;

DEFINE TABLE SCHEMAFULL repo_block;
DEFINE FIELD did ON TABLE repo_block TYPE string;
DEFINE FIELD cid ON TABLE repo_block TYPE string;
DEFINE FIELD repoRev ON TABLE repo_block TYPE string;
DEFINE FIELD size ON TABLE repo_block TYPE int;
DEFINE FIELD content ON TABLE repo_block TYPE bytes;
DEFINE INDEX repo_block_repo_rev_idx ON repo_block FIELDS repoRev, cid UNIQUE;

DEFINE TABLE SCHEMAFULL record;
DEFINE FIELD cid ON TABLE record TYPE string;
DEFINE FIELD did ON TABLE record TYPE string;
DEFINE FIELD collection ON TABLE record TYPE string;
DEFINE FIELD rkey ON TABLE record TYPE string;
DEFINE FIELD repoRev ON TABLE record TYPE option<string>;
DEFINE FIELD indexedAt ON TABLE record TYPE datetime;
DEFINE FIELD takedownRef ON TABLE record TYPE option<string>;
DEFINE INDEX record_did_cid_idx ON record FIELDS cid UNIQUE;
DEFINE INDEX record_did_collection_idx ON record FIELDS collection UNIQUE;
DEFINE INDEX record_repo_rev_idx ON record FIELDS repoRev UNIQUE;

DEFINE TABLE SCHEMAFULL blob;
DEFINE FIELD cid ON TABLE blob TYPE string;
DEFINE FIELD did ON TABLE blob TYPE string;
DEFINE FIELD mimeType ON TABLE blob TYPE string;
DEFINE FIELD size ON TABLE blob TYPE int;
DEFINE FIELD tempKey ON TABLE blob TYPE option<string>;
DEFINE FIELD width ON TABLE blob TYPE int;
DEFINE FIELD height ON TABLE blob TYPE int;
DEFINE FIELD createdAt ON TABLE blob TYPE datetime
    VALUE time::now() READONLY;
DEFINE INDEX blob_pkey ON blob FIELDS cid, did UNIQUE;
DEFINE INDEX blob_tempkey_idx ON blob FIELDS tempKey UNIQUE;

DEFINE TABLE SCHEMAFULL record_blob;
DEFINE FIELD blobCid ON TABLE record_blob TYPE string;
DEFINE FIELD recordUri ON TABLE record_blob TYPE string;
DEFINE FIELD did ON TABLE record_blob TYPE string;
