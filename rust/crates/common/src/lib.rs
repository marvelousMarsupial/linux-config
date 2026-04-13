//! Shared primitives for forge: roles, paths, state, atomic file ops, diffing.

pub mod atomic;
pub mod diff;
pub mod executor;
pub mod packages;
pub mod paths;
pub mod plan;
pub mod planner;
pub mod role;
pub mod run;
pub mod dev_config;
pub mod incus;
pub mod podman;
pub mod state;
pub mod theme;

pub use atomic::write_atomic;
pub use diff::unified_diff;
pub use executor::{execute, ActionResult};
pub use paths::{config_dir, home, state_file};
pub use plan::{Action, Plan, PlanSummary};
pub use planner::{build_plan, build_teardown_plan, DotfileSource, InMemoryDotfiles};
pub use role::{Role, RoleConfig, RolesFile};
pub use run::{checked, run, run_capture};
pub use state::State;
