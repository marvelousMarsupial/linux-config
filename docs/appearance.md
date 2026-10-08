# Appearance

[Documentation index](README.md)

The desktop uses the selected Base16 palette throughout. Summer Dusk remains the live selection; the other five palettes still render through the same templates.

- Desktop: base00 background, equal 6px gutters, 2px borders. Accent color identifies the focused window; inactive outlines use a quiet surface color.
- Foot: 11pt CaskaydiaCove, centered 12x10px padding, 8% background when tiled and at least 60% when floating in glass mode. Solid mode stays opaque; the beam cursor is steady.
- Fuzzel: the same font family at 12pt, ten rows, 26px row height, 1px outline and 8px corners. Selection and search accents come from the palette.
- Mako: matching typography, 14px padding, 1px outlines and 72% background opacity in glass mode.
- Neovim: rounded native popup borders, bounded completion menu, no filler tildes or accidental dashboard tabline. Prose wrapping is retained.
- Yazi: pane proportions 1:3:4 give the preview more room.
- Starship: two lines and a dollar prompt, readable git/language labels, normal path display.

The appearance gallery uses GTK 3. These styling changes add no services, blur, animations, or polling. No performance benchmark was performed; they adjust existing rendering and layout settings.

References reviewed for visual direction:

- [okapmna's minimal Sway setup](https://github.com/okapmna/sway-dotfiles-mint)
- [Lokesh Krishna's Catppuccin setup](https://github.com/lokesh-krishna/dotfiles/blob/main/catppuccin/readme.md)
- [Catppuccin's Fuzzel themes](https://github.com/catppuccin/fuzzel)

The implementation uses this repository's existing palette and installed software. Reload Sway with Super+Shift+c; open a new terminal and restart Neovim/Yazi to inspect their changes. GTK applications may need reopening too.

## Dark mode is the default

Fresh installs should use dark mode throughout, including KeePassXC. Preserve
this requirement when changing desktop themes or adding applications. Sway does
not require a full desktop environment, but toolkit preferences must be configured.

- GTK 3/4 settings and generated CSS use dark colors. `desktop-appearance` sets
  GNOME's `prefer-dark` preference and `Adwaita-dark` theme at Sway startup and reload,
  covering libadwaita and portal-aware apps.
- `environment.d/appearance.conf` sets `GTK_THEME=Adwaita:dark` using GTK's
  built-in Adwaita theme. The login shell exports it before starting Sway, and
  `sway-session` imports it into D-Bus and systemd. Keep this lightweight: no
  additional Qt theme plugins, theme managers or desktop environment are required.
  Set independent Qt application themes directly, as with KeePassXC below.
- KeePassXC's fresh config explicitly sets `GUI/ApplicationTheme=dark`. Existing
  app settings are preserved while the installer merges only the Dark theme.
- OpenSnitch uses its supported `global/theme=dark_amber.xml` setting. The repo
  includes `python3-qt-material` (about 455 KiB plus its font dependency), required
  by its own theme selector. No global Qt theme plugin or new service is added.
  Existing firewall preferences are preserved; only the UI theme is merged.
- Zen's theme sync explicitly requests dark system/content colors, including
  `prefers-color-scheme: dark`, while preserving unrelated profile preferences.
  The launcher repeats sync before launch once a default profile and generated
  colors exist. Fresh first launch relies on the desktop preference; later launches
  also have the explicit profile preferences.

The repo app launchers reapply the dark setting before each launch, including
Super+P, the application launcher and terminal commands using the installed PATH.
Running windows update on their next fresh launch; setup does not force-close
password databases or interrupt firewall prompts.

After changing toolkit environment defaults, log out/in and reopen apps so both Sway-launched
and activated applications inherit the settings. Reloading Sway alone cannot
change its inherited environment or the theme of already-running apps.
Apps with independent themes may need an app-specific setting. Websites must
support the dark preference or offer their own dark mode; this does not recolor
websites that only implement a light design.

Sources: [Debian Qt 6 GTK theme plugin](https://packages.debian.org/trixie/libs/qt6-gtk-platformtheme),
[KeePassXC theme setting](https://github.com/keepassxreboot/keepassxc/blob/2.7.10/src/core/Config.cpp),
[Mozilla color preferences](https://github.com/mozilla-firefox/firefox/blob/main/modules/libpref/init/StaticPrefList.yaml).

## Themes

Press **Super+G** (or open **Appearance** from the launcher) for a compact
picker showing only palette dots and a transparency switch. **Up/Down** moves
through palettes and the switch; **Left/Right** moves between them.
**Enter** on a palette applies it and closes. **Enter** or **Space** on the switch
toggles transparency immediately and keeps the popup open. **Escape** closes
without applying the palette being browsed. The dots show the desktop background, two surfaces, text, and two accents.
The larger first dot emphasizes the background instead of a rainbow of ANSI colors.
Mouse selection and the switch also work. Theme names appear only on hover.
The picker uses GTK 3 (`python3-gi`, `gir1.2-gtk-3.0`).

The command-line tools remain available:

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

Zen's browser UI follows the same palette: surfaces, text, tabs, search
highlights, buttons, and menus. Glass mode keeps its existing translucency
with palette-colored accents. Changing themes updates the default Zen profile;
**restart Zen** to load the new colors. Web pages keep their own styling.
No extension or background service is needed.

## Glass and drafting-grid wallpaper

The current glass setup uses a pin-dot wallpaper with matching 8% backgrounds
for Zen chrome, tiled Foot, and Fuzzel (including the info panel). Floating Foot
windows use at least 60% background opacity, including upload/save pickers;
text stays opaque. The popup guard updates opacity when floating changes.
Use **Appearance → Transparency** in the app launcher; restart Zen and reopen terminals afterward.
`wallpaper-texture dots` and `wallpaper-texture grid` select fixed-spacing
repeating patterns. Larger displays show more cells. Physical multi-monitor,
mixed-scale, and rotation checks remain pending; existing monitor profiles are
retained. See [glass mode and wallpaper](transparency-trial.md) for commands,
settings, validation, and rollback details.
