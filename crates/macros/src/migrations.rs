use peripheral_hub_storage::Migration;

/// Forward-only schema migrations for macro persistence.
pub const MACRO_MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "create_macros",
    sql: "CREATE TABLE macros (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL,
        profile_id TEXT,
        action_count INTEGER NOT NULL,
        schema_version INTEGER NOT NULL,
        data TEXT NOT NULL,
        updated_at INTEGER NOT NULL
    );",
}];
