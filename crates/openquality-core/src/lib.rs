//! # OpenQuality Core
//!
//! The core library for OpenQuality — a data quality and observability platform.
//!
//! ## Modules
//!
//! - [`types`] — All data types: `MonitorType` (10 variants), `ExpectationType` (17 variants),
//!   `Severity`, `Incident`, `MonitorResult`, and configuration enums
//! - [`monitor`] — 10 monitor implementations: freshness, volume, schema, distribution,
//!   correlation, uniqueness, referential integrity, custom SQL, ML drift, cost
//! - [`expectation`] — Expectation suites with 17 built-in assertion types and a runner
//! - [`alert`] — Alert channel trait with 5 implementations (stdout, JSON, Slack, PagerDuty, webhook)
//! - [`root_cause`] — Root cause analysis with causal inference (Granger, PCA, dimension isolation)
//! - [`stats`] — Statistical functions: KS test, JS divergence, chi-square, MAD, IQR, z-score, auto-thresholds
//! - [`profiler`] — Column profiling (null count, distinct count, quantiles, histograms)
//! - [`suggest`] — Suggestion engine that recommends expectations from profiles
//! - [`store`] — Storage trait (30+ methods) with record types
//! - [`error`] — Error types

pub mod alert;
pub mod error;
pub mod expectation;
pub mod monitor;
pub mod profiler;
pub mod root_cause;
pub mod stats;
pub mod store;
pub mod suggest;
pub mod types;

pub use error::{OpenQualityError, Result};
pub use types::*;

/// Convenience re-exports for common use cases.
pub mod prelude {
    pub use crate::alert::*;
    pub use crate::expectation::{BuiltinExpectations, ExpectationRunner};
    pub use crate::monitor::*;
    pub use crate::profiler::Profiler;
    pub use crate::root_cause::*;
    pub use crate::store::*;
    pub use crate::suggest::SuggestionEngine;
}
