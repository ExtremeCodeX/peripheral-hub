use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;

use crate::{Migration, StorageError};

/// Owned SQLite connection with a bounded busy timeout and foreign keys enabled.
///
/// A single connection is intentionally kept (no pool): the daemon performs all
/// persistence on its serialized manager worker, so contention is not modeled.
pub struct Database {
    connection: Connection,
}

impl Database {
    /// Open or create a file-backed database.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let connection = Connection::open(path)?;
        Self::configure(connection)
    }

    /// Open a private in-memory database, used by tests and ephemeral sessions.
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let connection = Connection::open_in_memory()?;
        Self::configure(connection)
    }

    fn configure(connection: Connection) -> Result<Self, StorageError> {
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.busy_timeout(Duration::from_secs(5))?;
        Ok(Self { connection })
    }

    /// Current `user_version`, or 0 for a fresh database.
    pub fn schema_version(&self) -> Result<u32, StorageError> {
        let version: i64 = self
            .connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;
        u32::try_from(version).map_err(|_| StorageError::InvalidSchemaVersion)
    }

    /// Apply every migration newer than the stored schema version, in order.
    ///
    /// Already-applied migrations are skipped. A database whose schema is newer
    /// than `migrations` is rejected rather than downgraded. Each migration runs
    /// in its own transaction so no migration partially applies.
    pub fn migrate(&mut self, migrations: &[Migration]) -> Result<(), StorageError> {
        for pair in migrations.windows(2) {
            if pair[0].version == 0 || pair[0].version >= pair[1].version {
                return Err(StorageError::InvalidMigrations);
            }
        }
        if migrations.first().is_some_and(|m| m.version == 0) {
            return Err(StorageError::InvalidMigrations);
        }
        let current = self.schema_version()?;
        let latest = migrations.iter().map(|m| m.version).max().unwrap_or(0);
        if current > latest {
            return Err(StorageError::SchemaNewer {
                found: current,
                supported: latest,
            });
        }
        for migration in migrations.iter().filter(|m| m.version > current) {
            let transaction = self.connection.transaction()?;
            transaction.execute_batch(migration.sql).map_err(|source| {
                StorageError::MigrationFailed {
                    version: migration.version,
                    name: migration.name,
                    source,
                }
            })?;
            transaction.pragma_update(None, "user_version", i64::from(migration.version))?;
            transaction.commit()?;
            tracing::info!(
                event = "schema_migrated",
                version = migration.version,
                name = migration.name
            );
        }
        Ok(())
    }

    /// Borrow the underlying connection for crate-local repositories.
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Mutably borrow the connection for transactional repositories.
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V1: &[Migration] = &[Migration {
        version: 1,
        name: "create_items",
        sql: "CREATE TABLE items (id TEXT PRIMARY KEY NOT NULL);",
    }];

    const V2: &[Migration] = &[
        Migration {
            version: 1,
            name: "create_items",
            sql: "CREATE TABLE items (id TEXT PRIMARY KEY NOT NULL);",
        },
        Migration {
            version: 2,
            name: "add_label",
            sql: "ALTER TABLE items ADD COLUMN label TEXT;",
        },
    ];

    #[test]
    fn migrations_apply_once_and_track_version() {
        let mut database = Database::open_in_memory().unwrap();
        assert_eq!(database.schema_version().unwrap(), 0);
        database.migrate(V1).unwrap();
        assert_eq!(database.schema_version().unwrap(), 1);
        // Re-running the same set is a no-op, not an error.
        database.migrate(V1).unwrap();
        assert_eq!(database.schema_version().unwrap(), 1);
        database.migrate(V2).unwrap();
        assert_eq!(database.schema_version().unwrap(), 2);
    }

    #[test]
    fn newer_schema_is_rejected_without_downgrade() {
        let mut database = Database::open_in_memory().unwrap();
        database.migrate(V2).unwrap();
        let error = database.migrate(V1).unwrap_err();
        assert!(matches!(
            error,
            StorageError::SchemaNewer {
                found: 2,
                supported: 1
            }
        ));
        assert_eq!(database.schema_version().unwrap(), 2);
    }

    #[test]
    fn invalid_migration_sets_are_rejected() {
        let mut database = Database::open_in_memory().unwrap();
        let duplicated = &[
            Migration {
                version: 1,
                name: "a",
                sql: "SELECT 1;",
            },
            Migration {
                version: 1,
                name: "b",
                sql: "SELECT 1;",
            },
        ];
        assert!(matches!(
            database.migrate(duplicated).unwrap_err(),
            StorageError::InvalidMigrations
        ));
    }

    #[test]
    fn failed_migration_rolls_back_and_preserves_version() {
        let mut database = Database::open_in_memory().unwrap();
        let broken = &[Migration {
            version: 1,
            name: "broken",
            sql: "CREATE TABLE t (id TEXT); CREATE TABLE t (id TEXT);",
        }];
        assert!(matches!(
            database.migrate(broken).unwrap_err(),
            StorageError::MigrationFailed { version: 1, .. }
        ));
        assert_eq!(database.schema_version().unwrap(), 0);
    }

    #[test]
    fn file_backed_database_persists_across_reopen() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("state.sqlite");
        {
            let mut database = Database::open(&path).unwrap();
            database.migrate(V1).unwrap();
        }
        let database = Database::open(&path).unwrap();
        assert_eq!(database.schema_version().unwrap(), 1);
    }
}
