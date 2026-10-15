# Desktop theme review

2026-10-15 — configuration review, not a live visual inspection. The sandbox
cannot access the user's compositor. No palette or geometry changes were made
during this review; the previously confirmed untinted tiled terminals remain.

## Existing coverage

The shared Base16 scheme already generates foot, Sway, fuzzel, Mako, swaylock,
GTK 3/4 CSS and Zen styles. The shell prompt inherits terminal ANSI colors.
Foot now uses FiraCode Nerd Font Mono 11. Sway and Mako use CaskaydiaCove Nerd
Font 11; fuzzel uses CaskaydiaCove at size 12. Fuzzel and Mako generally share
an 8 px radius and 1 px border.
No swaybar block is configured in the managed Sway config.

Neovim can use either generated Base16 colors or a named plugin theme.
Yazi uses bundled flavors for some schemes. Those paths can diverge from
custom palette adjustments, so selecting the same theme name does not guarantee
identical colors across all apps.

## Gaps to review next

- GTK settings use Sans 10 rather than the desktop's Nerd Font family.
- Compare FiraCode in Foot with CaskaydiaCove in Sway, fuzzel and Mako; decide
  whether those apps should use the same family.
- GTK uses Adwaita icons; fuzzel requests hicolor. A common installed icon theme
  should be chosen before changing either setting.
- Sway borders are 2 px, versus 1 px for fuzzel/Mako. Sway inner gaps are 6 px;
  popup margins/padding serve different purposes and need visual comparison.
- The status popup uses generic monospace at size 18.
- Qt theming has no target in the shared theme configuration.
- Check the named nvim/Yazi themes against custom palette edits.

## Contrast

Calculated from the current Catppuccin Mocha palette on opaque backgrounds:

| Pair | Contrast |
| --- | --- |
| Main text / base background | 11.34:1 |
| Main text / raised background | 12.14:1 |
| Subtle text / base background | 4.44:1 |
| Blue accent / base background | 7.79:1 |

Subtle text is slightly below a 4.5:1 target for small text. Transparency means
actual contrast also depends on wallpaper and windows beneath it; these numbers
do not establish live glass-mode readability. Compare launcher placeholders,
notification bodies and floating terminals against both quiet and busy content.

## Next visual pass

Open foot, fuzzel, Notifications, nvim, Yazi, a GTK picker and Zen under the same
palette. Compare typography, text readability, accents and border weights before
choosing changes. Notification access is described in [notifications](notifications.md).
