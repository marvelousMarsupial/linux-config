use anyhow::{bail, Result};

use crate::cli::ThemeCmd;

pub fn dispatch(sub: ThemeCmd) -> Result<()> {
    match sub {
        ThemeCmd::List => bail!("theme list: not yet implemented"),
        ThemeCmd::Current => bail!("theme current: not yet implemented"),
        ThemeCmd::Apply { name } => bail!("theme apply {name}: not yet implemented"),
        ThemeCmd::Preview { name } => bail!("theme preview {name}: not yet implemented"),
    }
}
