#[macro_export]
macro_rules! query {
    (storage: $storage:expr, table: $table:expr $(, $op:ident = $val:expr)* $(,)?) => {{
        let mut builder = $crate::QueryBuilder::new($storage, $table);
        $(
            builder = builder.$op($val);
        )*
        builder.execute()
    }};
}

#[macro_export]
macro_rules! query_project {
    (storage: $storage:expr, table: $table:expr, project = $project:expr $(, $op:ident = $val:expr)* $(,)?) => {{
        let mut builder = $crate::QueryBuilder::new($storage, $table);
        $(
            builder = builder.$op($val);
        )*
        builder.project($project)?.execute()
    }};
}
