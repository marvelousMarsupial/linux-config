use anyhow::{bail, Result};
use forge_common::Role;

pub fn run(_role: Role, _dry_run: bool, _diff: bool) -> Result<()> {
    bail!("forge setup: not yet implemented");
}

pub fn switch(_role: Role) -> Result<()> {
    bail!("forge switch: not yet implemented");
}

pub fn teardown(_full: bool) -> Result<()> {
    bail!("forge teardown: not yet implemented");
}

pub fn diff(_role: Role) -> Result<()> {
    bail!("forge diff: not yet implemented");
}
