//! # OpenQuality Store
//!
//! Storage implementations for the OpenQuality platform.
//!
//! - [`PgStore`] — PostgreSQL-backed implementation of the `Store` trait (via sqlx).
//!   Supports all 12 tables with migrations, serde_json-based monitor type storage,
//!   and full CRUD for workspaces, users, monitors, incidents, data sources, and more.
//!   Connected via `DATABASE_URL` environment variable.
//!
//! Also provides [`InMemoryStore`](openquality_core::store::InMemoryStore) in the core crate for testing.

pub mod pg;

pub use pg::PgStore;
