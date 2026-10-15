# Desktop plan

[Documentation index](README.md)

The desktop vision and the ideas not yet scheduled. Open tasks with priority
live in [TODO](TODO.md#desktop); this file keeps the reasoning and options.

Current setup: Debian, Sway, Foot, Fuzzel, Neovim, Yazi and Zen. The goal is a
desktop that looks and feels like one deliberate system while staying minimal.

## Decisions so far

- **Launcher:** stay on Fuzzel. It runs only when called, and it is small,
  maintained and fits the minimal goal.
- **Lua config managers** (zerb, Dotty, rdm, sm): skip. They add a layer for
  little gain on one machine. Git plus the repo's installer is enough.
- **Custom shells:** not needed now. For reference, Quickshell is Qt with QML
  plus JavaScript; AGS is GTK with TypeScript, JSX and CSS on GJS, with Astal
  libraries for battery, network, audio, tray, apps and notifications. Both
  are persistent processes; 50 to 200 MB is a planning ballpark, not a
  measurement.
- **Notifications:** Mako with the themed history viewer on Super+Shift+N.
  SwayNotificationCenter stays an option if a real panel with do-not-disturb
  is wanted. See [notifications](notifications.md).
- **Palette:** one Base16 source rendered through templates. Wallust (palette
  from wallpaper) was considered and not chosen. See [appearance](appearance.md).
- **Font:** CaskaydiaCove Nerd Font everywhere. FiraCode was tried in Foot
  and removed to keep one family.
- **Dark preference:** published through GTK settings and the portal; Zen
  requests dark content. Done; see [appearance](appearance.md#dark-mode-is-the-default).

## Fuzzel alternatives (reference)

- **Minimal:** wmenu (plainer than Fuzzel), bemenu, tofi (development quiet).
- **Heavier:** rofi (now native Wayland), walker (Go), anyrun (Rust).
- **Stalled:** wofi.

## Typography and geometry

- One corner radius, border width and gap size across Sway, Fuzzel and Mako
  where supported.
- Try **SwayFX**, a Sway fork with compatible configuration, for rounded
  corners, blur, shadows and dimming inactive windows.

## Apps that ignore the theme

- A GTK theme for file pickers, plus qt6ct or Kvantum if a Qt app needs it.
- Matching cursor and icon themes.
- Optionally darkman for scheduled light and dark switching.

## Consistent keys

- hjkl movement everywhere: Sway, Neovim, Yazi, zsh vi mode and the browser.
- The same close or quit key across tools.
- Sway modes that mirror the Neovim leader key.

## Polish start to finish

- Theme the boot loader and configure quiet boot.
- Consider greetd with tuigreet and the matching palette. Reconcile this with
  the disk-passphrase-only boot and automatic Sway login task before choosing
  a login flow.
- Theme the lock screen with swaylock-effects or gtklock: palette, font, clock.
- Mako rules with distinct urgency colors; mute noisy apps.

## Sway features to try

Already in use: `--to-code` bindings, touchpad gestures, scratchpad plus sticky
for notes, floating rules for dialogs.

- **Marks:** label windows, jump to them and swap by mark.
- **Binding modes:** launch mode, system mode and leader-key style.
- **Keyboard layouts:** US plus US International with a toggle, or a compose
  key, for accents in Spanish, French, Italian and Portuguese.
- **for_window rules:** assign apps to workspaces.
- **Smart gaps and smart borders,** tabbed default layout, workspace back and
  forth.
- **Swaybar:** hide until modifier, tray, JSON status with click events. No
  bar is configured today; the status panel covers the basics on demand.
- **IPC:** a small TypeScript or Go daemon for auto-tiling, workspace renaming
  or per-window layouts, following the repository's language conventions.

## Extras

- Switch the system locale to Spanish or Italian for passive input.
- A bar module showing course progress, daily language input or a Latin word
  of the day.
