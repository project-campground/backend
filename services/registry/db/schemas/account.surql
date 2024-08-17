DEFINE TABLE SCHEMAFULL account;
DEFINE FIELD email ON TABLE account TYPE string
    ASSERT string::is::email($value);
DEFINE FIELD password ON TABLE account TYPE string;
DEFINE FIELD created_at ON TABLE account TYPE datetime
    VALUE time::now() READONLY;
DEFINE FIELD email_confirmed_at ON TABLE account TYPE option<datetime>;