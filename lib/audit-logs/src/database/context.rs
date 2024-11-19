use si_data_pg::{PgPool, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;

use super::AuditDatabaseConfig;

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum AuditDatabaseContextError {
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
}

type Result<T> = std::result::Result<T, AuditDatabaseContextError>;

/// The context used for communicating with and setting up the audit database.
#[derive(Debug, Clone)]
pub struct AuditDatabaseContext {
    pg_pool: PgPool,
}

impl AuditDatabaseContext {
    /// Creates an [`AuditDatabaseContext`] from an [`AuditDatabaseConfig`].
    #[instrument(level = "info", name = "audit.context.from_config", skip_all)]
    pub async fn from_config(config: &AuditDatabaseConfig) -> Result<Self> {
        Ok(Self {
            pg_pool: PgPool::new(&config.pg).await?,
        })
    }

    /// Returns a reference to the [`PgPool`].
    pub fn pg_pool(&self) -> &PgPool {
        &self.pg_pool
    }
}
