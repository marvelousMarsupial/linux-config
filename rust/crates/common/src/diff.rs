//! Unified text diffs for `forge diff` / `--diff` output.

use similar::{ChangeTag, TextDiff};

/// Return a unified diff between `old` and `new`. Empty string means no changes.
pub fn unified_diff(old: &str, new: &str, label: &str) -> String {
    let diff = TextDiff::from_lines(old, new);
    if diff.ratio() == 1.0 {
        return String::new();
    }
    let mut out = String::new();
    out.push_str(&format!("--- {label} (old)\n+++ {label} (new)\n"));
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        out.push_str(sign);
        out.push_str(change.value());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_when_identical() {
        assert_eq!(unified_diff("a\nb\n", "a\nb\n", "f"), "");
    }

    #[test]
    fn shows_diff_for_changes() {
        let d = unified_diff("a\nb\n", "a\nc\n", "f");
        assert!(d.contains("-b"));
        assert!(d.contains("+c"));
    }

    #[test]
    fn includes_label() {
        let d = unified_diff("a\n", "b\n", "zshrc");
        assert!(d.contains("zshrc"));
    }
}
