use anyhow::{bail, Result};

use crate::cli::DevCmd;

pub fn dispatch(sub: DevCmd) -> Result<()> {
    match sub {
        DevCmd::Build { image } => bail!("dev build {image}: not yet implemented"),
        DevCmd::Up { name } => bail!("dev up {name}: not yet implemented"),
        DevCmd::Ls => bail!("dev ls: not yet implemented"),
        DevCmd::Attach { name } => bail!("dev attach {name}: not yet implemented"),
        DevCmd::Exec { name, cmd: _ } => bail!("dev exec {name}: not yet implemented"),
        DevCmd::Down { name } => bail!("dev down {name}: not yet implemented"),
        DevCmd::Rm { name } => bail!("dev rm {name}: not yet implemented"),
    }
}
