#[derive(Debug, Clone)]
/// PostgreSQL pool model used by the service-level database context.
pub struct PostgresPool {
    pool: sqlx::PgPool,
}

impl PostgresPool {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub fn inner(&self) -> &sqlx::PgPool {
        &self.pool
    }

    pub fn into_inner(self) -> sqlx::PgPool {
        self.pool
    }
}