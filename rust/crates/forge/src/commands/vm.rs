use anyhow::{bail, Result};

use crate::cli::VmCmd;

pub fn dispatch(sub: VmCmd) -> Result<()> {
    match sub {
        VmCmd::Create { profile } => bail!("vm create {profile}: not yet implemented"),
        VmCmd::Ls => bail!("vm ls: not yet implemented"),
        VmCmd::Start { name } => bail!("vm start {name}: not yet implemented"),
        VmCmd::Stop { name } => bail!("vm stop {name}: not yet implemented"),
        VmCmd::Shell { name } => bail!("vm shell {name}: not yet implemented"),
    }
}
