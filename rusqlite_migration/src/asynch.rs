use tokio_rusqlite::Connection as AsyncConnection;

use crate::errors::Result;
use crate::{Migrations, SchemaVersion, M};

/// Adapter to make `Migrations` available in an async context.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsyncMigrations {
    migrations: Migrations<'static>,
}

impl AsyncMigrations {
    /// Adapt a [Migrations](crate::Migrations) instance for use in an asynchronous context.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rusqlite_migration::{Migrations, AsyncMigrations, M};
    ///
    /// let migrations = AsyncMigrations::new(vec![
    ///     M::up("CREATE TABLE animals (name TEXT);"),
    ///     M::up("CREATE TABLE food (name TEXT);"),
    /// ]);
    /// ```
    pub fn new(ms: Vec<M<'static>>) -> Self {
        Self {
            migrations: Migrations::new(ms),
        }
    }

    /// Asynchronous version of the same method in the [Migrations](crate::Migrations::current_version) struct.
    ///
    /// # Example
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// use rusqlite_migration::{Migrations, AsyncMigrations, M, SchemaVersion};
    /// use std::num::NonZeroUsize;
    ///
    /// let mut conn = tokio_rusqlite::Connection::open_in_memory().await.unwrap();
    ///
    /// let migrations = AsyncMigrations::new(vec![
    ///     M::up("CREATE TABLE animals (name TEXT);"),
    ///     M::up("CREATE TABLE food (name TEXT);"),
    /// ]);
    ///
    /// assert_eq!(SchemaVersion::NoneSet, migrations.current_version(&conn).await.unwrap());
    ///
    /// // Go to the latest version
    /// migrations.to_latest(&mut conn).await.unwrap();
    ///
    /// assert_eq!(SchemaVersion::Inside(NonZeroUsize::new(2).unwrap()), migrations.current_version(&conn).await.unwrap());
    /// # })
    /// ```
    pub async fn current_version(&self, async_conn: &AsyncConnection) -> Result<SchemaVersion> {
        let m = self.migrations.clone();
        async_conn.call(move |conn| m.current_version(conn)).await
    }

    /// Asynchronous version of the same method in the [Migrations](super::Migrations::to_latest) struct.
    ///
    /// # Example
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// use rusqlite_migration::{Migrations, AsyncMigrations, M};
    /// let mut conn = tokio_rusqlite::Connection::open_in_memory().await.unwrap();
    ///
    /// let migrations = AsyncMigrations::new(vec![
    ///     M::up("CREATE TABLE animals (name TEXT);"),
    ///     M::up("CREATE TABLE food (name TEXT);"),
    /// ]);
    ///
    /// // Go to the latest version
    /// migrations.to_latest(&mut conn).await.unwrap();
    ///
    /// // You can then insert values in the database
    /// conn.call(|conn| conn.execute("INSERT INTO animals (name) VALUES (?)", ["dog"])).await.unwrap();
    /// conn.call(|conn| conn.execute("INSERT INTO food (name) VALUES (?)", ["carrot"])).await.unwrap();
    /// # });
    /// ```
    pub async fn to_latest(&self, async_conn: &mut AsyncConnection) -> Result<()> {
        let m = self.migrations.clone();
        async_conn.call(move |conn| m.to_latest(conn)).await
    }

    /// Asynchronous version of the same method in the [Migrations](crate::Migrations::to_version) struct.
    ///
    /// # Example
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// use rusqlite_migration::{Migrations, AsyncMigrations, M};
    /// let mut conn = tokio_rusqlite::Connection::open_in_memory().await.unwrap();
    /// let migrations = AsyncMigrations::new(vec![
    ///     // 0: version 0, before having run any migration
    ///     M::up("CREATE TABLE animals (name TEXT);").down("DROP TABLE animals;"),
    ///     // 1: version 1, after having created the “animals” table
    ///     M::up("CREATE TABLE food (name TEXT);").down("DROP TABLE food;"),
    ///     // 2: version 2, after having created the food table
    /// ]);
    ///
    /// migrations.to_latest(&mut conn).await.unwrap(); // Create all tables
    ///
    /// // Go back to version 1, i.e. after running the first migration
    /// migrations.to_version(&mut conn, 1).await;
    /// conn.call(|conn| conn.execute("INSERT INTO animals (name) VALUES (?)", ["dog"])).await.unwrap();
    /// conn.call(|conn| conn.execute("INSERT INTO food (name) VALUES (?)", ["carrot"]).unwrap_err()).await;
    ///
    /// // Go back to an empty database
    /// migrations.to_version(&mut conn, 0).await;
    /// conn.call(|conn| conn.execute("INSERT INTO animals (name) VALUES (?)", ["cat"]).unwrap_err()).await;
    /// conn.call(|conn| conn.execute("INSERT INTO food (name) VALUES (?)", ["milk"]).unwrap_err()).await;
    /// # })
    /// ```
    pub async fn to_version(&self, async_conn: &mut AsyncConnection, version: usize) -> Result<()> {
        let m = self.migrations.clone();
        async_conn
            .call(move |conn| m.to_version(conn, version))
            .await
    }

    /// Asynchronous version of the same method in the [Migrations](crate::Migrations::validate) struct.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[cfg(test)]
    /// mod tests {
    ///
    ///     // … Other tests …
    ///
    ///     #[tokio::test]
    ///     fn migrations_test() {
    ///         assert!(migrations.validate().await.is_ok());
    ///     }
    /// }
    /// ```
    pub async fn validate(&self) -> Result<()> {
        let mut async_conn = AsyncConnection::open_in_memory().await?;
        self.to_latest(&mut async_conn).await
    }
}
