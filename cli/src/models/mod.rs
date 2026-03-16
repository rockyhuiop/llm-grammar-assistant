//! Data models for grammar checking.

pub mod config;
pub mod edit;

pub use config::{CloudConfig, CloudProvider, Configuration, LocalConfig, OperatingMode};
pub use edit::{CheckResult, Edit, EditCategory, LlmResponse, ProcessingMetadata};
