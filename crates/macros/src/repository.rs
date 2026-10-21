use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use peripheral_hub_storage::{Database, rusqlite::params};

use crate::{MACRO_MIGRATIONS, MACRO_SCHEMA_VERSION, Macro, MacroError, MacroId, MacroSummary};

/// Validated persistence facade over the `macros` table.
///
/// Writes serialize the whole validated macro into a single row; reads
/// re-validate and reject unknown future schema versions rather than guessing.
pub struct MacroRepository {
    database: Database,
}

impl MacroRepository {
    /// Adopt an existing database and apply macro migrations.
    pub fn open(mut database: Database) -> Result<Self, MacroError> {
        database.migrate(MACRO_MIGRATIONS)?;
        Ok(Self { database })
    }

    /// Open a private in-memory repository (tests and ephemeral sessions).
    pub fn open_in_memory() -> Result<Self, MacroError> {
        Self::open(Database::open_in_memory()?)
    }

    /// Open or create a file-backed repository.
    pub fn open_file(path: impl AsRef<Path>) -> Result<Self, MacroError> {
        Self::open(Database::open(path)?)
    }

    /// Insert or replace the macro identified by [`Macro::id`].
    pub fn save(&mut self, macro_value: &Macro) -> Result<(), MacroError> {
        macro_value.validate()?;
        let data = serde_json::to_string(macro_value)
            .map_err(|_| MacroError::Invalid("macro serialization failed"))?;
        self.database.connection().execute(
            "INSERT INTO macros (id, name, profile_id, action_count, schema_version, data, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 profile_id = excluded.profile_id,
                 action_count = excluded.action_count,
                 schema_version = excluded.schema_version,
                 data = excluded.data,
                 updated_at = excluded.updated_at",
            params![
                macro_value.id.as_str(),
                &macro_value.name,
                macro_value.profile.as_ref().map(|profile| profile.as_str()),
                macro_value.actions.len() as i64,
                i64::from(MACRO_SCHEMA_VERSION),
                &data,
                unix_seconds(),
            ],
        )?;
        Ok(())
    }

    /// Fetch and re-validate one macro.
    pub fn get(&self, id: &MacroId) -> Result<Macro, MacroError> {
        let mut statement = self
            .database
            .connection()
            .prepare("SELECT schema_version, data FROM macros WHERE id = ?1")?;
        let mut rows = statement.query(params![id.as_str()])?;
        let Some(row) = rows.next()? else {
            return Err(MacroError::NotFound);
        };
        let schema_version =
            u16::try_from(row.get::<_, i64>(0)?).map_err(|_| MacroError::UnsupportedVersion)?;
        if schema_version != MACRO_SCHEMA_VERSION {
            return Err(MacroError::UnsupportedVersion);
        }
        let data: String = row.get(1)?;
        let macro_value: Macro =
            serde_json::from_str(&data).map_err(|_| MacroError::InvalidJson)?;
        macro_value.validate()?;
        Ok(macro_value)
    }

    /// List macro summaries ordered by name then id.
    pub fn list(&self) -> Result<Vec<MacroSummary>, MacroError> {
        let mut statement = self
            .database
            .connection()
            .prepare("SELECT id, name, profile_id, action_count FROM macros ORDER BY name, id")?;
        let mut rows = statement.query([])?;
        let mut summaries = Vec::new();
        while let Some(row) = rows.next()? {
            let id = MacroId::new(row.get::<_, String>(0)?)?;
            let name: String = row.get(1)?;
            let profile = row
                .get::<_, Option<String>>(2)?
                .map(|value| {
                    peripheral_hub_profiles::ProfileId::new(value)
                        .map_err(|_| MacroError::Invalid("stored profile link is invalid"))
                })
                .transpose()?;
            let action_count = usize::try_from(row.get::<_, i64>(3)?)
                .map_err(|_| MacroError::Invalid("stored action count is invalid"))?;
            summaries.push(MacroSummary {
                id,
                name,
                action_count,
                profile,
            });
        }
        Ok(summaries)
    }

    /// Delete one macro; missing ids are reported, not silently ignored.
    pub fn delete(&mut self, id: &MacroId) -> Result<(), MacroError> {
        let affected = self
            .database
            .connection()
            .execute("DELETE FROM macros WHERE id = ?1", params![id.as_str()])?;
        if affected == 0 {
            return Err(MacroError::NotFound);
        }
        Ok(())
    }

    /// Number of stored macros.
    pub fn count(&self) -> Result<u64, MacroError> {
        let count: i64 =
            self.database
                .connection()
                .query_row("SELECT COUNT(*) FROM macros", [], |row| row.get(0))?;
        u64::try_from(count).map_err(|_| MacroError::Invalid("macro count is invalid"))
    }
}

fn unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}
