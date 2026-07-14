//! # OpenQuality Auth
//!
//! Authentication and authorization for the OpenQuality platform.
//!
//! - [`jwt`] — JWT token creation, validation, and refresh (using jsonwebtoken + HS256)
//! - [`password`] — Argon2 password hashing and verification
//! - [`api_key`] — API key generation (SHA-256 hashed, hex-encoded) and validation
//! - [`rbac`] — Role-based access control with 5 roles: Owner, Admin, Editor, Member, Viewer

pub mod api_key;
pub mod jwt;
pub mod password;
pub mod rbac;

pub use api_key::*;
pub use jwt::*;
pub use password::*;
pub use rbac::*;
