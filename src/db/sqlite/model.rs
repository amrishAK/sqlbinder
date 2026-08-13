#[derive(Debug, Clone)]
/// SQLite pool model used by the service-level database context.
pub struct SqlitePool {
    pool: sqlx::SqlitePool,
}

impl SqlitePool {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    pub fn inner(&self) -> &sqlx::SqlitePool {
        &self.pool
    }

    pub fn into_inner(self) -> sqlx::SqlitePool {
        self.pool
    }
}  