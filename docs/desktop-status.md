# Desktop customization status — 2026-10-07

Historical snapshot of the glass/fullscreen work, with later changes noted below.
For current repository behavior and verification boundaries, use [current state](state.md).
Statements about active services are evidence from that earlier session.

## Accepted and installed

- **One glass switch:** Super+Ctrl+G, or `glass-mode on|off|toggle|status`.
  Restart Zen and reopen terminals after switching. Palette selection is preserved.
- **Light mode for lower-spec machines:** glass off uses a solid wallpaper and
  opaque system surfaces, skips texture generation, and removes the managed Zen
  glass import. No performance benchmark has been performed.
- **Pin-dot wallpaper:** current live selection is `dots` with Summer Dusk.
  `wallpaper-texture list`, a style name, or `next` changes it. Grid/dots use a
  120×120 tile with 24-pixel spacing and stronger anchors every five cells.
  Larger displays show more cells instead of stretching the pattern.
- **Matching surfaces:** normal Zen chrome and tiled Foot use an 8% base00 background.
  Floating Foot uses at least 60% opacity for readability over webpages.
  Fuzzel launchers and the centered clock/network/power info panel use about 8%.
  Text stays opaque. GTK and Mako backgrounds retain their stronger 72% tint.
- **Zen search:** a subtle resting-field surface, clear outer container, consistent
  expanded-panel background, and softer divider. Browser overlays can blur browser
  content; standard Sway does not blur desktop backgrounds.
- **Zen fullscreen appearance:** a locally generated wallpaper copy is painted
  behind fullscreen chrome, preserving the dot appearance. Web pages stay opaque.
  This is a local background replica, not visibility through to other applications.
- **Fullscreen popups:** `sway-popup-guard.service` is active and starts with the
  Sway session. The current revision always temporarily exits fullscreen for newly opened
  windows, including those reported visible, and centers/floats them up to
  800×550 logical pixels and 65% of the workspace, then restores fullscreen after
  the intervening windows close or leave the workspace. Clicking the tiled
  background leaves floating windows above it. With other tiled windows present,
  their original layout can become visible during the interruption.
- **Dependencies:** `pass` removed from `packages.txt`; KeePassXC retained.
  This did not uninstall the existing pass package or migrate passwords.

## Pending and limitations

- **Foot native fullscreen:** background still appears solid because Sway hides
  the wallpaper and Foot has no equivalent local-image background setting.
- **Cross-monitor behavior:** user reports dragging a floater above fullscreen
  from another monitor. Headless IPC moves did not reproduce that exact path.
  Physical mouse dragging, mixed DPI/scaling, rotation, hotplug, and the saved
  hardware monitor profiles still need validation. Existing monitor settings kept.
- **Password shortcut (updated 2026-10-08):** Super+P now opens/focuses KeePassXC
  and toggles its focused window into the scratchpad. Browser integration defaults
  and the native messaging manifest are installed. Database selection/creation,
  Zen extension installation/pairing and live autofill remain pending. No password
  database contents were read or migrated. See [passwords](passwords.md).
- **Popup state:** restoration is in memory. Stopping/restarting the guard during
  an interruption loses its pending restoration; Super+F restores it manually.
  Stop it for the current session with `systemctl --user stop sway-popup-guard`.
- Zen upload and floating-picker visibility from fullscreen were confirmed by
  the user on 2026-10-08. Save/overwrite, global fullscreen, and multiple physical
  outputs still need application/hardware tests. No compositor patch or fullscreen
  layout replacement was installed.

## Security review

The focused review found no high-severity issue introduced by glass mode. No
plugin was installed; CSS imports are local. Running real Zen processes checked
had no remote-debugging/system-access/sandbox-disable override, and no Zen listener
was observed. Known temporary screenshots were restricted to the current user.
The earlier review identified persistent clipboard history and no configured Sway
idle-lock timeout. Transparency can reveal background information during screen
sharing; blur is not redaction. No OpenSnitch prompts were approved or its rules
changed as part of this desktop work. See [the scoped review](glass-security-review.md).

## Validation

- Syntax/whitespace checks for changed scripts, CSS, and docs.
- Generated dot tile dimensions, repeat spacing, and Sway tile configuration.
- Screenshots comparing Zen, Foot, and the info panel.
- Isolated headless Sway tests with real Foot windows: new popup size/floating/focus,
  multiple popups, and fullscreen restoration after the final popup closes.
- Native GTK dialog with guard active: parent stays genuinely fullscreen and the
  dialog remains visible after focusing the background.
- Isolated Zen fullscreen: root state, local wallpaper, 8% tint, and screenshot.
- Live user service activation; systemd unit verification. These checks do not
  establish physical multi-monitor correctness or a complete host security audit.

## Commits and related docs

- `08298c1`: shared glass switch.
- `725e56c`: search refinement and sharper wallpaper.
- `be8187c`: wallpaper variants, repeating grids, matching surfaces, and docs.
- This status accompanies the fullscreen popup/Zen appearance follow-up and pass
  dependency removal. Unrelated pre-existing working-tree changes are excluded.

[Glass mode and wallpaper](transparency-trial.md) ·
[Floating windows and fullscreen](window-behavior.md) ·
[Appearance](appearance.md)
