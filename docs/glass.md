# Glass mode and wallpaper

[Documentation index](README.md)

Glass mode makes the desktop translucent over a patterned wallpaper. Off is
the default on a fresh install: a solid wallpaper, opaque surfaces and no
texture generation. The browser is not part of glass mode.

## One switch

`glass-mode on`, `glass-mode off`, `glass-mode toggle` and `glass-mode status`
control desktop glass. The same switch sits in the
**Appearance** popup on **Super+G**. State lives in
`~/.config/theme/glass-mode`; no file means off.

Switching re-applies the current palette. Reopen terminals afterward; reopen
launchers and GTK apps as needed. Running apps stay open and
the palette is kept.

## What becomes translucent

- Tiled Foot windows are untinted. Floating Foot windows, including upload and
  save pickers, use at least 60% opacity so text stays readable over pages. The
  popup guard updates this when a window changes between floating and tiled.
  See [window behavior](window-behavior.md#floating-foot-opacity).
- Fuzzel launchers and the status panel use about 8%. The status panel
  switches to 85% when the focused workspace has a window under it.
- Mako and GTK backgrounds stay about 72% opaque. Text is always opaque. The
  lock screen stays solid.

Standard Sway has no compositor blur, so surfaces are tinted, not blurred.

## Wallpaper styles

**Dots** is the default: a seamless 120 by 120 pixel tile with a point every
24 pixels and a stronger anchor every five cells, drawn in the palette's base00.
Sway tiles it, so a larger display shows more cells instead of stretching them.

```sh
wallpaper-texture list
wallpaper-texture grid
wallpaper-texture next
wallpaper-texture frost --output ~/frost-preview.png
```

Styles: `dots`, `grid`, `woven`, `linen`, `frost`, `paper` and `brushed`. Grid
also tiles. The other textures render to the largest active output and use
`fill`. The choice is stored in `~/.config/theme/wallpaper-style`; running the
command without a style regenerates the current one, and `--output` previews
without changing anything. Every theme apply regenerates the wallpaper from
base00 and keeps the style. With glass off the command sets a solid color.

Sway hides the wallpaper behind a native fullscreen window; Foot has no
image-background setting, so a fullscreen window looks solid.

## Implementation and rollback

Theme sources live in `dotfiles/.config/theme/`. The switch writes the state
file and re-applies the palette; the templates read the state to pick opaque
or translucent colors.

`glass-mode off` is the rollback. The older snapshot tool from the first
transparency trial is retired; any backups it left live in
`~/.local/state/linux-config/transparency-trial`.

Privacy: translucent surfaces can reveal what is behind them during screen
sharing or to someone nearby. Blur is not redaction. The security notes are in
the [security audit](security-audit.md#glass-mode-review-2026-10-07).

## Pending checks

Physical multi-monitor validation is pending: compare pattern spacing on a
smaller and a larger monitor, and check mixed scaling and rotation. Generated
tile size, repeated dot pixels and the Sway tile configuration were verified on
a 1280 by 800 virtual display.

