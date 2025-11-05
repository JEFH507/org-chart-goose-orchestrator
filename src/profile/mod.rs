// Profile module - Role-based configuration and governance
//
// This module implements the profile system that enables zero-touch deployment
// for users. When a user signs in via OIDC, their entire Goose environment is
// auto-configured based on their role profile.

pub mod schema;
pub mod validator;
pub mod signer;

pub use schema::*;
pub use validator::ProfileValidator;
pub use signer::ProfileSigner;
