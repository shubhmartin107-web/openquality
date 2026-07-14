pub mod correlation;
pub mod cost;
pub mod custom_sql;
pub mod distribution;
pub mod freshness;
pub mod ml_drift;
pub mod referential;
pub mod schema;
pub mod uniqueness;
pub mod volume;

pub use correlation::CorrelationMonitor;
pub use cost::CostMonitor;
pub use custom_sql::CustomSQLMonitor;
pub use distribution::DistributionMonitor;
pub use freshness::FreshnessMonitor;
pub use ml_drift::MLDriftMonitor;
pub use referential::ReferentialIntegrityMonitor;
pub use schema::SchemaMonitor;
pub use uniqueness::UniquenessMonitor;
pub use volume::VolumeMonitor;

use crate::error::Result;
use crate::types::MonitorResult;

pub trait Monitor {
    fn run(&self) -> Result<MonitorResult>;
    fn name(&self) -> &str;
}
