//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Settings service — settings persistence orchestration at the application layer.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::application::errors::ApplicationError;
use crate::application::utils::name_is_available;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig};
use crate::error::{EngineError, Result};

#[derive(Clone)]
pub struct SettingsService {
    storage: Arc<Storage>,
}

impl SettingsService {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    pub fn get_settings(&self) -> Result<AppSettings> {
        self.storage.get_settings()
    }

    pub fn update_settings<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut AppSettings) -> Result<T>,
    {
        self.storage.update_settings(f)
    }

    /// Add a connection. Refuses a name already used by another connection;
    /// two cards the user cannot tell apart must not exist.
    pub fn add_connection(
        &self,
        connection: LlmProviderConfig,
    ) -> std::result::Result<AppSettings, ApplicationError> {
        self.storage
            .update_settings(|settings| {
                Self::ensure_connection_name_available(settings, &connection.name, None)?;
                settings.connections.push(connection.clone());
                Ok(settings.clone())
            })
            .map_err(Self::settings_error_to_application)
    }

    /// Apply `edit` to the connection with `id`, refusing a name already used
    /// by a different connection. The connection keeps its own name.
    pub fn update_connection<F>(
        &self,
        id: &str,
        edit: F,
    ) -> std::result::Result<(LlmProviderConfig, bool, bool), ApplicationError>
    where
        F: FnOnce(&mut LlmProviderConfig),
    {
        self.storage
            .update_settings(|settings| {
                let is_narrator = settings.narration_connection_id == id;
                let is_quantifier = settings.quantifier_connection_id == id;
                let (name, updated) = {
                    let connection = settings
                        .find_connection_mut(id)
                        .ok_or_else(|| EngineError::Config("Connection not found".to_string()))?;
                    edit(connection);
                    (connection.name.clone(), connection.clone())
                };
                Self::ensure_connection_name_available(settings, &name, Some(id))?;
                Ok((updated, is_narrator, is_quantifier))
            })
            .map_err(Self::settings_error_to_application)
    }

    fn connection_name_available(
        settings: &AppSettings,
        name: &str,
        except_id: Option<&str>,
    ) -> bool {
        name_is_available(
            settings
                .connections
                .iter()
                .map(|connection| (connection.id.as_str(), connection.name.as_str())),
            name,
            except_id,
        )
    }

    fn ensure_connection_name_available(
        settings: &AppSettings,
        name: &str,
        except_id: Option<&str>,
    ) -> Result<()> {
        if Self::connection_name_available(settings, name, except_id) {
            Ok(())
        } else {
            Err(EngineError::Validation(format!(
                "A connection named '{}' already exists",
                name.trim()
            )))
        }
    }

    fn settings_error_to_application(error: EngineError) -> ApplicationError {
        match error {
            EngineError::Validation(message) => ApplicationError::Validation(message),
            other => ApplicationError::Engine(other),
        }
    }
}
