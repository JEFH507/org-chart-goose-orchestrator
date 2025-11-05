// Policy engine module for RBAC/ABAC enforcement
// Phase 5 Workstream C

pub mod engine;

pub use engine::{PolicyEngine, PolicyContext, PolicyError};
