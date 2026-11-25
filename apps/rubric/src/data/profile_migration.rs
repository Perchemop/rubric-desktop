//! Profile schema migration system.
//!
//! Handles version-to-version migrations using pure, deterministic functions.
//! Each migration takes JSON `Value` and returns a migrated `Value`.
//!
//! Migrations are pure functions - no I/O, no side effects, fully testable.

use serde_json::Value;

use super::DataError;
use crate::domain::profile::CURRENT_PROFILE_SCHEMA_VERSION;

/// Type alias for a migration function.
/// Takes a JSON Value and returns a migrated JSON Value.
pub type MigrationFn = fn(Value) -> Result<Value, DataError>;

/// Migration registry: maps (from_version, to_version) to migration function.
/// We store migrations as adjacent version pairs and chain them as needed.
static MIGRATIONS: &[(u32, MigrationFn)] = &[
    // v1 -> v2: Add new fields with defaults
    // This is a placeholder - add actual migrations here as the schema evolves
];

/// Get the migration function from `from_version` to `from_version + 1`.
pub fn get_migration(from_version: u32) -> Option<MigrationFn> {
    MIGRATIONS
        .iter()
        .find(|(v, _)| *v == from_version)
        .map(|(_, f)| *f)
}

/// Apply migrations to bring a JSON value from `from_version` to `to_version`.
///
/// Migrations are chained: if going from v1 to v3, we apply v1->v2 then v2->v3.
pub fn migrate(mut value: Value, from_version: u32, to_version: u32) -> Result<Value, DataError> {
    if from_version == to_version {
        return Ok(value);
    }

    if from_version > to_version {
        return Err(DataError::UnsupportedSchema {
            found: from_version,
            supported: to_version,
        });
    }

    // Apply migrations in sequence
    let mut current_version = from_version;
    while current_version < to_version {
        let migration = get_migration(current_version).ok_or_else(|| {
            DataError::ProfileError(format!(
                "No migration path from v{} to v{}",
                current_version, to_version
            ))
        })?;

        value = migration(value)?;
        current_version += 1;
    }

    // Update schema version in the result
    if let Value::Object(ref mut obj) = value {
        obj.insert(
            "schema_version".to_string(),
            Value::Number(to_version.into()),
        );
    }

    Ok(value)
}

/// Migrate a profile JSON file content to the current schema version.
///
/// This is the main entry point called by the profile store when loading.
pub fn migrate_profile(value: Value) -> Result<Value, DataError> {
    // Extract the schema version from the JSON
    let from_version = match value.get("schema_version") {
        Some(v) => v.as_u32().unwrap_or(1),
        None => 1, // Default assumption for files without version field
    };

    if from_version > CURRENT_PROFILE_SCHEMA_VERSION {
        return Err(DataError::UnsupportedSchema {
            found: from_version,
            supported: CURRENT_PROFILE_SCHEMA_VERSION,
        });
    }

    if from_version == CURRENT_PROFILE_SCHEMA_VERSION {
        return Ok(value);
    }

    migrate(value, from_version, CURRENT_PROFILE_SCHEMA_VERSION)
}

/// Default v1 -> v2 migration: adds `is_predefined` field.
fn migrate_v1_to_v2(mut value: Value) -> Result<Value, DataError> {
    if let Value::Object(ref mut obj) = value {
        // Add missing fields with defaults
        if !obj.contains_key("is_predefined") {
            obj.insert("is_predefined".to_string(), Value::Bool(false));
        }
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::panel::PanelLayout;
    use crate::domain::profile::Profile;
    use serde_json::json;

    #[test]
    fn migrate_same_version_returns_unchanged() {
        let value = json!({"schema_version": 1});
        let result = migrate(value.clone(), 1, 1).unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn migrate_newer_version_returns_error() {
        let value = json!({"schema_version": 3});
        let result = migrate(value, 3, 1);
        assert!(result.is_err());
    }

    #[test]
    fn migrate_profile_handles_missing_schema_version() {
        let value = json!({"name": "test"});
        let result = migrate_profile(value);
        // Should default to v1 and succeed if v1 is current
        assert!(result.is_ok());
    }

    #[test]
    fn migrate_profile_refuses_unsupported_schema() {
        let value = json!({
            "schema_version": 99,
            "name": "test"
        });
        let result = migrate_profile(value);
        assert!(matches!(result, Err(DataError::UnsupportedSchema { .. })));
    }

    #[test]
    fn profile_schema_version_is_1() {
        assert_eq!(CURRENT_PROFILE_SCHEMA_VERSION, 1);
    }

    // Test for when we add v2 migration
    #[test]
    fn migrate_v1_to_v2_adds_is_predefined() {
        // This test validates the v1->v2 migration logic
        let v1_value = json!({
            "schema_version": 1,
            "name": "Test Profile",
            "id": "00000000-0000-0000-0000-000000000001",
        });

        // Simulate the v1->v2 migration
        let migrated = migrate_v1_to_v2(v1_value.clone()).unwrap();

        assert_eq!(migrated["is_predefined"], json!(false));
    }

    #[test]
    fn migrate_chains_multiple_versions() {
        // Test that migrations can be chained
        // For now, with v1 as current, this is a no-op test
        let value = json!({
            "schema_version": 1,
            "name": "Test"
        });
        let result = migrate(value, 1, CURRENT_PROFILE_SCHEMA_VERSION).unwrap();
        assert_eq!(result["schema_version"], json!(1));
    }
}
