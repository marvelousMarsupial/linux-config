# Glass mode and wallpaper

[Documentation index](README.md)

Glass mode makes the desktop and Zen's browser chrome translucent over a
patterned wallpaper. Off is the default on a fresh install: a solid wallpaper,
opaque surfaces, no texture generation and no managed Zen import.

## One switch

`glass-mode on`, `glass-mode off`, `glass-mode toggle` and `glass-mode status`
control desktop and Zen glass together. The same switch sits in the
**Appearance** popup on **Super+G**. State lives in
`~/.config/theme/glass-mode`; no file means off.

Switching re-applies the current palette. Restart Zen and reopen terminals
afterward; reopen launchers and GTK apps as needed. Running apps stay open and
the palette is kept.

## What becomes translucent

- Zen chrome uses an 8% base00 tint. The floating sidebar uses 38% with 8 px
  blur; the expanded search uses 24% with 14 px blur. Web pages stay opaque:
  the preference that would let pages be transparent is forced off. Only Zen's
  empty-tab placeholder is cleared.
- Tiled Foot windows are untinted. Floating Foot windows, including upload and
  save pickers, use at least 60% opacity so text stays readable over pages. The
  popup guard updates this when a window changes between floating and tiled.
  See [window behavior](window-behavior.md#floating-foot-opacity).
- Fuzzel launchers and the status panel use about 8%.
- Mako and GTK backgrounds stay about 72% opaque. Text is always opaque. The
  lock screen stays solid.

Standard Sway has no compositor blur. Zen blurs its own content behind its
overlays using the Natsumi clip workaround; attribution is in
`licenses/natsumi-MIT.txt`.

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

Sway hides the wallpaper behind a native fullscreen window. Zen paints a local
copy of the same image behind its chrome in that state; Foot has no equivalent
setting, so a fullscreen terminal looks solid.

## Implementation and rollback

Theme sources live in `dotfiles/.config/theme/`; Zen CSS in
`dotfiles/.config/zen/userChrome.css`. The switch installs a managed import in
the default Zen profile's `userChrome.css` and sets three preferences in its
`user.js`: legacy stylesheet support on, Linux chrome transparency to match the
mode, and page transparency off. Every theme apply also runs `zen-theme-sync`,
which copies the generated palette CSS into the profile. Its import precedes
the glass import so glass wins for translucency. Other CSS and preferences are
preserved.

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

Sources: [Zen preferences](https://docs.zen-browser.app/guides/about-config-flags),
[Transparent Zen](https://github.com/sameerasw/zen-themes/tree/main/TransparentZen),
[Natsumi sidebar CSS](https://github.com/Psychosoc1al/natsumi/blob/main/natsumi/modules/sidebars.css).
