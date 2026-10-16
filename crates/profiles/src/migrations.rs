use peripheral_hub_storage::Migration;

/// Forward-only schema migrations for profile persistence.
pub const PROFILE_MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "create_profiles",
    sql: "CREATE TABLE profiles (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL,
        device_model TEXT,
        schema_version INTEGER NOT NULL,
        data TEXT NOT NULL,
        updated_at INTEGER NOT NULL
    );",
}];
