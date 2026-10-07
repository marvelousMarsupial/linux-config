# Glass mode security review — 2026-10-07

Reviewed the glass scripts, theme renderer and templates, Zen CSS and selected
live preferences, running Zen arguments/environment, listeners, permissions,
clipboard capture, and lock configuration. This is a scoped review, not a full
host penetration test, dependency/CVE audit, or firewall verification.

The accepted search and wallpaper refinements are committed as `725e56c`.

## Findings

- **Medium, existing: clipboard persistence.** The installed Sway config runs
  `wl-paste --watch cliphist store -max-items 10`. Copied secrets can remain in
  clipboard history. Glass mode neither creates nor resolves this behavior.
- **Medium, existing: no inactivity lock configured in Sway.** Swayidle locks
  before sleep, and manual locking exists, but its invocation has no idle
  timeout. This check does not exclude an independent external locking service.
- **Low, glass privacy tradeoff:** translucent surfaces can reveal underlying
  terminal/document content in screen shares or to someone nearby. Blur is not
  redaction. Reopen affected applications after disabling glass; Zen requires
  a restart. The generated swaylock configuration remains solid.
- **Low, corrected: review screenshot permissions.** Known temporary screenshots
  from this work were mode 0664 in /tmp. They are now 0600, as is the test-control
  metadata file. Test profile directories are 0700. No screenshot content was
  inspected for secrets during this audit.

## Positive checks and limits

- The two observed running Zen processes had no Marionette/remote-debugging
  arguments, system-access environment override, or content-sandbox-disable
  environment override. No Zen TCP listener appeared in `ss -lntup` at review.
  The real profile had no explicit debug or security-disable setting among the
  preference keys checked. These are targeted checks, not proof of every
  effective runtime security preference.
- Live `zen-themes.json` is empty: no downloaded mod runs for glass mode. CSS
  imports are local; the upstream attribution URL is only a comment. The Python
  scripts do not fetch remote code or use shell=True/eval. Theme hook commands
  are local trusted configuration and use subprocess argument lists.
- Glass manages three appearance preferences, leaving web-page transparency
  disabled. No sandbox/TLS/certificate preference is changed by these scripts.
- CSS does not hide identity, permission, or tracking controls. The only hidden
  content surface is conditioned on Zen's own empty-tab root attribute. Contrast
  can still vary with the underlying background and future browser updates.
- Home, the active Zen profile, runtime directory, and trial-backup directory
  are 0700. Local CSS/configuration must remain trusted: modifying browser UI
  styling can disguise security indicators. Other processes running as this
  user can already modify these files; glass introduces no elevated privilege.
- No automatic package upgrades, clipboard deletion, firewall change, or idle
  policy change was performed during this review.

Sources: [Zen's appearance preferences](https://docs.zen-browser.app/guides/about-config-flags)
and [Mozilla remote-agent security](https://firefox-source-docs.mozilla.org/remote/Security.html).
