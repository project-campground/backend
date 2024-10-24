DEFINE TABLE SCHEMAFULL account;
DEFINE FIELD email ON TABLE account TYPE string
    ASSERT string::is::email($value);
DEFINE FIELD password ON TABLE account TYPE string;
DEFINE FIELD createdAt ON TABLE account TYPE datetime
    VALUE time::now() READONLY;
DEFINE FIELD emailConfirmedAt ON TABLE account TYPE option<datetime>;