# Appearance

[Documentation index](README.md)

The desktop uses the selected Base16 palette throughout. Summer Dusk remains the live selection; the other five palettes still render through the same templates.

- Desktop: base00 background, equal 6px gutters, 2px borders. Accent color identifies the focused window; inactive outlines use a quiet surface color.
- Foot: existing 11pt CaskaydiaCove font, centered 12x10px padding, opaque base01 surface and steady beam cursor.
- Fuzzel: the same font family at 12pt, ten rows, 26px row height, 1px outline and 8px corners. Selection and search accents come from the palette.
- Mako: matching typography, 14px padding, 1px outlines and opaque surfaces.
- Neovim: rounded native popup borders, bounded completion menu, no filler tildes or accidental dashboard tabline. Prose wrapping is retained.
- Yazi: pane proportions 1:3:4 give the preview more room.
- Starship: two lines and a dollar prompt, readable git/language labels, normal path display.

These changes add no packages, plugins, services, blur, animations, or polling. No performance benchmark was performed; they adjust existing rendering and layout settings.

References reviewed for visual direction:

- [okapmna's minimal Sway setup](https://github.com/okapmna/sway-dotfiles-mint)
- [Lokesh Krishna's Catppuccin setup](https://github.com/lokesh-krishna/dotfiles/blob/main/catppuccin/readme.md)
- [Catppuccin's Fuzzel themes](https://github.com/catppuccin/fuzzel)

The implementation uses this repository's existing palette and installed software. Reload Sway with Super+Shift+c; open a new terminal and restart Neovim/Yazi to inspect their changes. GTK applications may need reopening too.

## Themes

```sh
theme list
theme preview catppuccin-mocha
theme apply catppuccin-mocha
theme current
```

Six schemes remain available, including TokyoNight, Catppuccin, Kanagawa,
Gruvbox, One Half Dark, and Summer Dusk. Python 3.11+, `python3-jinja2`, and
`python3-yaml` are required. Schemes, templates, and target definitions live
in `dotfiles/.config/theme/` — one source of truth, no build step.

Theme application renders every template before writing and runs configured
application reload hooks. Existing Foot processes are never signaled; open a new terminal
for terminal colors. Reload Sway and restart open editors to finish applying colors.
An absent application may report that its reload is unavailable.
