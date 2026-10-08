# Glass mode and wallpaper

[Documentation index](README.md)

## One switch

`glass-mode on`, `glass-mode off`, `glass-mode toggle`, and `glass-mode status`
control desktop and Zen glass together. Open **Appearance** from the app launcher or press **Super+G**. Use **Up/Down** to reach a palette or the transparency switch.
**Enter** on the switch toggles transparency immediately and leaves the popup
open; **Enter** on a palette applies the palette and closes. **Escape** closes.
Restart Zen and reopen terminals after changing modes; reopen launchers and GTK
applications as needed. Running applications stay open and the palette is retained.
State is stored in `~/.config/theme/glass-mode`; no state file means glass is off.

## Accepted appearance

- Zen chrome follows the system palette for text, surfaces and accents in both
  solid and glass modes. Restart Zen after selecting a new theme.
- Zen glass chrome uses an 8% base00 tint. Floating sidebar tint is 38%; expanded
  search tint is 24%, with 8px sidebar blur and 14px search blur. The resting search field has a
  faint edge. Its outer container is clear to avoid a ghost pill when opened.
- Tiled Foot uses an 8% base00 background in glass mode. Floating Foot uses at
  least 60%, updated by the popup guard when floating changes. Text remains opaque.
  Glass off restores a solid base01 background. [Activation](window-behavior.md#floating-foot-opacity)
- Fuzzel launchers and the centered info panel use base00 with alpha `14`
  (about 8%). Glass off restores solid base01. Text remains opaque.
- Mako notification backgrounds and GTK backgrounds remain about 72% opaque.
- Real web pages remain opaque. Only Zen's explicit empty-tab placeholder is
  cleared. Full-window opacity stays at 1; the lock screen remains solid.

Standard Sway has no compositor backdrop blur. Zen can blur browser content
behind its own overlays using the conditional Natsumi clip workaround. This
is not desktop blur. Attribution is in `licenses/natsumi-MIT.txt`.

## Wallpaper styles

The accepted live selection is **dots**, on the Summer Dusk palette.

```sh
wallpaper-texture list
wallpaper-texture dots
wallpaper-texture grid
wallpaper-texture next
wallpaper-texture frost --output /tmp/frost-preview.png
```

Styles: `woven`, `linen`, `frost`, `paper`, `brushed`, `grid`, and `dots`.
Selection is stored in `~/.config/theme/wallpaper-style`. With no selection,
`woven` is used. Running the command without a style regenerates the current
selection. A preview output does not change the selection or desktop.
Theme application regenerates the wallpaper from base00 and preserves the style.

Grid and dots use a seamless **120×120-pixel repeating tile**: cell spacing is
24 pixels, with a stronger mark every five cells. Sway uses `tile`, so larger
outputs show more cells rather than stretching them. Other textures use `fill`
and render to the largest active output resolution.

## Validation and pending hardware test

Verified the generated tile size, repeated dot pixels, and generated Sway `tile`
configuration on the current 1280×800 virtual display. Browser, terminal, and
info popup were compared in screenshots. Source syntax and whitespace checks pass.

**Physical multi-monitor validation remains pending.** Keep the existing monitor
configuration. On hardware, check the saved monitor layout, connect/disconnect,
Sway reload, and palette/glass changes. Compare a smaller and larger monitor:
both should show the same pattern spacing with more cells on the larger screen.
Also check mixed output scaling and rotation; physical size on different-DPI
panels has not been calibrated or verified.

## Implementation and rollback

Theme sources live in `dotfiles/.config/theme/`; Zen CSS lives in
`dotfiles/.config/zen/userChrome.css`. The switch installs a managed local import
and updates only the three appearance preferences in the default Zen profile.
Every theme apply also runs `zen-theme-sync`, which installs the generated
`system-theme.css`, the glass palette and glass styling in that profile.
Its managed import precedes the glass import, so the existing glass appearance
wins for translucency. Other CSS and unrelated preferences are preserved.
Web-page transparency stays disabled. Other CSS and preferences are retained.

`transparency-trial` is an older exact-file snapshot rollback utility, not the
routine mode switch. Its original backups remain in
`~/.local/state/linux-config/transparency-trial`. Later config edits or palette
changes can conflict with those snapshots. Use `glass-mode off` for normal use.

See [the focused security review](glass-security-review.md) for privacy tradeoffs
and existing clipboard/idle-lock findings.

Sources reviewed: [Zen preferences](https://docs.zen-browser.app/guides/about-config-flags),
[Transparent Zen](https://github.com/sameerasw/zen-themes/tree/main/TransparentZen),
[Natsumi sidebar CSS](https://github.com/Psychosoc1al/natsumi/blob/main/natsumi/modules/sidebars.css),
and the installed `sway-output(5)` manual for tiling behavior.

## Fullscreen behavior

Zen uses a local wallpaper replica behind fullscreen chrome because Sway hides
the desktop wallpaper in that state. Restart Zen after updating the CSS. See
[floating windows and fullscreen](window-behavior.md) for stock popup behavior, limitations, and tests.
