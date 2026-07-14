pub mod chi_square;
pub mod iqr;
pub mod js_divergence;
pub mod ks_test;
pub mod threshold;
pub mod zscore;

pub use chi_square::chi_square_test;
pub use iqr::iqr_outliers;
pub use js_divergence::js_divergence;
pub use ks_test::ks_test;
pub use threshold::{ThresholdMethod, auto_threshold};
pub use zscore::{modified_zscore, zscore};
