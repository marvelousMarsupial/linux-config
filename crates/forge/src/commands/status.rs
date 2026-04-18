use anyhow::Result;
use forge_common::{state_file, State};
use owo_colors::OwoColorize;

pub fn show() -> Result<()> {
    let path = state_file()?;
    let state = State::load(&path)?;

    cliclack::intro("forge status".bold().to_string())?;
    cliclack::log::info(format!(
        "Role:    {}",
        state
            .role
            .map(|r| r.to_string())
            .unwrap_or_else(|| "none".into())
    ))?;
    cliclack::log::info(format!(
        "Theme:   {}",
        state.theme.unwrap_or_else(|| "none".into())
    ))?;
    cliclack::log::info(format!(
        "Applied: {}",
        state.applied_at.unwrap_or_else(|| "never".into())
    ))?;
    cliclack::outro("ok")?;
    Ok(())
}
