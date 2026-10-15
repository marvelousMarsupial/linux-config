# Appearance

[Documentation index](README.md)

The desktop uses one Base16 palette and one font everywhere. Summer Dusk is the
default palette; the other five render through the same templates.

## Typography and geometry

CaskaydiaCove Nerd Font is the only font the repo installs. Setup fetches it
from the Nerd Fonts release when `fc-match` does not already resolve it.

| Surface | Setting |
| --- | --- |
| Foot | CaskaydiaCove Nerd Font Mono 11, centered 12x10 px padding, steady beam cursor. |
| Sway | CaskaydiaCove Nerd Font 11 for titles; 6 px inner gaps, no outer gaps, 2 px borders. |
| Fuzzel, status panel | CaskaydiaCove Nerd Font 12 (18 for the status panel), ten rows, 26 px rows, 1 px outline, 8 px corners. |
| Mako | CaskaydiaCove Nerd Font 11, 14 px padding, 1 px outline. |
| Surfingkeys | CaskaydiaCove Nerd Font with monospace fallbacks inside the browser. |
| GTK | Sans 10 for dialogs; Adwaita icons. |
| Neovim | Rounded native popup borders, bounded completion menu, no filler tildes. |
| Yazi | Pane proportions 1:3:4 so the preview gets more room. |
| Starship | Two lines, a dollar prompt, readable git and language labels. |

Known inconsistencies, tracked in [TODO](TODO.md): Sway borders are 2 px while
Fuzzel and Mako use 1 px; GTK dialogs use a proportional font and Adwaita icons
while Fuzzel requests hicolor; Qt has no theme target. Named Neovim and Yazi
themes for some palettes can drift from custom palette edits, so the same palette
name does not guarantee identical colors in every app.

Contrast on the Catppuccin Mocha palette, opaque backgrounds:

| Pair | Contrast |
| --- | --- |
| Main text on base background | 11.3:1 |
| Main text on raised background | 12.1:1 |
| Subtle text on base background | 4.4:1 |
| Blue accent on base background | 7.8:1 |

Subtle text is just under the 4.5:1 target for small text. In glass mode the
real contrast also depends on what is behind a window.

## Dark mode is the default

Fresh installs use dark mode throughout. Sway has no desktop environment, so
toolkit preferences are set explicitly and kept lightweight.

- GTK 3/4 settings and generated CSS use dark colors. `desktop-appearance` sets
  GNOME's `prefer-dark` preference and the `Adwaita-dark` theme at Sway startup
  and on reload, which covers libadwaita and portal-aware apps.
- `environment.d/appearance.conf` sets `GTK_THEME=Adwaita:dark`. The login
  shell exports it before starting Sway, and `sway-session` imports it into
  D-Bus and systemd. No Qt theme plugin, theme manager or desktop environment
  is required.
- KeePassXC gets `GUI/ApplicationTheme=dark`. OpenSnitch gets
  `global/theme=dark_amber.xml`, which needs `python3-qt-material`. The config
  installer merges only those keys into existing settings. The repo launchers
  reapply them before each launch.
- Zen's theme sync requests dark system and content colors, including
  `prefers-color-scheme: dark`, while preserving unrelated profile preferences.
  Websites must implement the preference themselves; Dark Reader covers the rest.

After changing toolkit environment defaults, log out and back in and reopen
apps. Reloading Sway cannot change its inherited environment or the theme of
already running apps.

Sources: [KeePassXC theme setting](https://github.com/keepassxreboot/keepassxc/blob/2.7.10/src/core/Config.cpp),
[Mozilla color preferences](https://github.com/mozilla-firefox/firefox/blob/main/modules/libpref/init/StaticPrefList.yaml).

## Themes

Press **Super+G**, or open **Appearance** from the launcher, for a compact
picker showing palette dots and a transparency switch. **Up/Down** moves through
palettes and the switch, **Left/Right** moves between them. **Enter** on a
palette applies it and closes. **Enter** or **Space** on the switch toggles
glass mode immediately and keeps the popup open. **Escape** closes without
applying the palette being browsed. The dots show the background, two surfaces,
text and two accents; names appear on hover. The picker uses GTK 3.

The command-line tools:

```sh
theme list
theme preview catppuccin-mocha
theme apply catppuccin-mocha
theme current
```

Six schemes: TokyoNight Moon, Catppuccin Mocha, Kanagawa, Gruvbox Dark Hard,
One Half Dark and Summer Dusk. `python3-jinja2` and `python3-yaml` are
required. Schemes, templates and target definitions live in
`dotfiles/.config/theme/`, one source of truth with no build step.

Applying a theme renders every template before writing anything, then runs the
configured reload hooks: the wallpaper is regenerated, Mako reloads, and the
Zen profile is synced. Existing Foot windows are never signaled; open a new
terminal for new colors. Reload Sway and restart open editors to finish.
Zen loads the new colors on restart.

Glass mode and wallpaper styles are described in [glass mode](glass.md).
