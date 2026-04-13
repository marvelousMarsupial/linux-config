//! Thin wrappers around subprocess execution.

use anyhow::{anyhow, Context, Result};
use duct::cmd;

/// Run a command, inheriting stdio. Fail if the command exits non-zero.
pub fn run(argv: &[&str]) -> Result<()> {
    let (head, tail) = argv
        .split_first()
        .ok_or_else(|| anyhow!("empty argv"))?;
    cmd(*head, tail)
        .run()
        .with_context(|| format!("running `{}`", argv.join(" ")))?;
    Ok(())
}

/// Run a command and capture stdout as a `String`. Fail if it exits non-zero.
pub fn run_capture(argv: &[&str]) -> Result<String> {
    let (head, tail) = argv
        .split_first()
        .ok_or_else(|| anyhow!("empty argv"))?;
    let out = cmd(*head, tail)
        .read()
        .with_context(|| format!("running `{}`", argv.join(" ")))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_echo() {
        let out = run_capture(&["echo", "hello"]).unwrap();
        assert_eq!(out.trim(), "hello");
    }

    #[test]
    fn run_true_ok() {
        assert!(run(&["true"]).is_ok());
    }

    #[test]
    fn run_false_errors() {
        assert!(run(&["false"]).is_err());
    }

    #[test]
    fn empty_argv_errors() {
        assert!(run(&[]).is_err());
    }
}
