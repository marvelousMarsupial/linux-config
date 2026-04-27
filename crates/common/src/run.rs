//! Thin wrappers around subprocess execution.
//!
//! `checked` is the one you want when a subprocess failure should abort
//! the caller with a useful error. It captures stderr and surfaces it in
//! the error message, so you see *why* podman/apt blew up instead of
//! just "exit status 1". `run` and `run_capture` stream stdio to the
//! user's terminal — use them for interactive commands.

use anyhow::{anyhow, Context, Result};
use duct::cmd;
use std::process::Command;

/// Run a command, inheriting stdio. Fail if the command exits non-zero.
pub fn run(argv: &[&str]) -> Result<()> {
    let (head, tail) = argv.split_first().ok_or_else(|| anyhow!("empty argv"))?;
    cmd(*head, tail)
        .run()
        .with_context(|| format!("running `{}`", argv.join(" ")))?;
    Ok(())
}

/// Run a command and capture stdout as a `String`. Fail if it exits non-zero.
pub fn run_capture(argv: &[&str]) -> Result<String> {
    let (head, tail) = argv.split_first().ok_or_else(|| anyhow!("empty argv"))?;
    let out = cmd(*head, tail)
        .read()
        .with_context(|| format!("running `{}`", argv.join(" ")))?;
    Ok(out)
}

/// Run a subprocess silently, capturing stdout+stderr. On non-zero exit
/// the error message includes the command line and the first ~800 bytes
/// of stderr — enough to diagnose typical apt/podman failures without
/// drowning the caller in output.
pub fn checked(argv: &[&str]) -> Result<String> {
    let (head, tail) = argv.split_first().ok_or_else(|| anyhow!("empty argv"))?;
    let out = Command::new(head)
        .args(tail)
        .output()
        .with_context(|| format!("spawning `{}`", argv.join(" ")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let snippet = stderr.chars().take(800).collect::<String>();
        anyhow::bail!(
            "`{}` exited {}: {}",
            argv.join(" "),
            out.status.code().unwrap_or(-1),
            snippet.trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
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

    #[test]
    fn checked_captures_stdout() {
        let s = checked(&["echo", "hi"]).unwrap();
        assert_eq!(s.trim(), "hi");
    }

    #[test]
    fn checked_surfaces_stderr_on_failure() {
        // `sh -c 'echo oops >&2; exit 2'` emits to stderr and exits non-zero.
        let err = checked(&["sh", "-c", "echo oops >&2; exit 2"]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("oops"));
        assert!(msg.contains("exited 2"));
    }
}
