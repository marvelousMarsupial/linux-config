//! Shared primitives for forge: roles, paths, state, atomic file ops, diffing.

pub mod atomic;
pub mod diff;
pub mod paths;
pub mod role;
pub mod run;
pub mod state;

pub use atomic::write_atomic;
pub use diff::unified_diff;
pub use paths::{config_dir, home, state_file};
pub use role::{Role, RoleConfig, RolesFile};
pub use run::{run, run_capture};
pub use state::State;
