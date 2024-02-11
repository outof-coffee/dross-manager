pub trait SqlModel {
    fn to_sql_insert(&self) -> String;
    fn generate_sql_create_table() -> String;

    fn table_name() -> String;
}