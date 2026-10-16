use rusqlite::Error as SqliteError;

/// One forward-only schema migration, identified by a strictly increasing version.
///
/// `sql` runs inside a single transaction together with the `user_version`
/// bump, so a failing migration never partially applies.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    /// Strictly increasing positive version; zero is reserved for "no schema".
    pub version: u32,
    /// Short stable name used in diagnostics and errors.
    pub name: &'static str,
    /// DDL/DML executed as one batch. Must be idempotent only through version gating.
    pub sql: &'static str,
}

/// Structured persistence failures; no user data or SQL text is included.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Underlying SQLite error, kept for local diagnostics only.
    #[error("sqlite error: {0}")]
    Sqlite(#[from] SqliteError),
    /// Migration versions were duplicated or not strictly ascending.
    #[error("migration versions must be unique and strictly ascending")]
    InvalidMigrations,
    /// A migration's transaction failed and was rolled back.
    #[error("migration {version} ({name}) failed")]
    MigrationFailed {
        /// Version of the failing migration.
        version: u32,
        /// Name of the failing migration.
        name: &'static str,
        /// Rolled-back SQLite error.
        #[source]
        source: SqliteError,
    },
    /// The on-disk schema is newer than this build understands.
    #[error("schema version {found} is newer than supported {supported}")]
    SchemaNewer {
        /// Version found in the database file.
        found: u32,
        /// Highest version known to this build.
        supported: u32,
    },
    /// The stored `user_version` cannot be represented as a `u32`.
    #[error("stored schema version is invalid")]
    InvalidSchemaVersion,
}
