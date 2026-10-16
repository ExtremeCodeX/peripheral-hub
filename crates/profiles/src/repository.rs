use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use peripheral_hub_device_sdk::DeviceId;
use peripheral_hub_storage::{Database, rusqlite::params};

use crate::{
    PROFILE_MIGRATIONS, PROFILE_SCHEMA_VERSION, Profile, ProfileError, ProfileId, ProfileSummary,
};

/// Validated persistence facade over the `profiles` table.
///
/// Writes serialize the whole validated profile into a single row; reads
/// re-validate and reject unknown future schema versions rather than guessing.
pub struct ProfileRepository {
    database: Database,
}

impl ProfileRepository {
    /// Adopt an existing database and apply profile migrations.
    pub fn open(mut database: Database) -> Result<Self, ProfileError> {
        database.migrate(PROFILE_MIGRATIONS)?;
        Ok(Self { database })
    }

    /// Open a private in-memory repository (tests and ephemeral sessions).
    pub fn open_in_memory() -> Result<Self, ProfileError> {
        Self::open(Database::open_in_memory()?)
    }

    /// Open or create a file-backed repository.
    pub fn open_file(path: impl AsRef<Path>) -> Result<Self, ProfileError> {
        Self::open(Database::open(path)?)
    }

    /// Insert or replace the profile identified by [`Profile::id`].
    pub fn save(&mut self, profile: &Profile) -> Result<(), ProfileError> {
        profile.validate()?;
        let data = serde_json::to_string(profile)
            .map_err(|_| ProfileError::Invalid("profile serialization failed"))?;
        self.database.connection().execute(
            "INSERT INTO profiles (id, name, device_model, schema_version, data, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 device_model = excluded.device_model,
                 schema_version = excluded.schema_version,
                 data = excluded.data,
                 updated_at = excluded.updated_at",
            params![
                profile.id.as_str(),
                &profile.name,
                profile.device_model.as_ref().map(DeviceId::as_str),
                i64::from(PROFILE_SCHEMA_VERSION),
                &data,
                unix_seconds(),
            ],
        )?;
        Ok(())
    }

    /// Fetch and re-validate one profile.
    pub fn get(&self, id: &ProfileId) -> Result<Profile, ProfileError> {
        let mut statement = self
            .database
            .connection()
            .prepare("SELECT schema_version, data FROM profiles WHERE id = ?1")?;
        let mut rows = statement.query(params![id.as_str()])?;
        let Some(row) = rows.next()? else {
            return Err(ProfileError::NotFound);
        };
        let schema_version =
            u16::try_from(row.get::<_, i64>(0)?).map_err(|_| ProfileError::UnsupportedVersion)?;
        if schema_version != PROFILE_SCHEMA_VERSION {
            return Err(ProfileError::UnsupportedVersion);
        }
        let data: String = row.get(1)?;
        let profile: Profile =
            serde_json::from_str(&data).map_err(|_| ProfileError::InvalidJson)?;
        profile.validate()?;
        Ok(profile)
    }

    /// List profile summaries ordered by name then id.
    pub fn list(&self) -> Result<Vec<ProfileSummary>, ProfileError> {
        let mut statement = self
            .database
            .connection()
            .prepare("SELECT id, name, device_model FROM profiles ORDER BY name, id")?;
        let mut rows = statement.query([])?;
        let mut summaries = Vec::new();
        while let Some(row) = rows.next()? {
            let id = ProfileId::new(row.get::<_, String>(0)?)?;
            let name: String = row.get(1)?;
            let device_model = row
                .get::<_, Option<String>>(2)?
                .map(DeviceId::new)
                .transpose()
                .map_err(|_| ProfileError::Invalid("stored device model is invalid"))?;
            summaries.push(ProfileSummary {
                id,
                name,
                device_model,
            });
        }
        Ok(summaries)
    }

    /// Delete one profile; missing ids are reported, not silently ignored.
    pub fn delete(&mut self, id: &ProfileId) -> Result<(), ProfileError> {
        let affected = self
            .database
            .connection()
            .execute("DELETE FROM profiles WHERE id = ?1", params![id.as_str()])?;
        if affected == 0 {
            return Err(ProfileError::NotFound);
        }
        Ok(())
    }

    /// Number of stored profiles.
    pub fn count(&self) -> Result<u64, ProfileError> {
        let count: i64 =
            self.database
                .connection()
                .query_row("SELECT COUNT(*) FROM profiles", [], |row| row.get(0))?;
        u64::try_from(count).map_err(|_| ProfileError::Invalid("profile count is invalid"))
    }
}

fn unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}
