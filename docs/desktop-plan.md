# Desktop plan

Recorded: 2026-10-15. This is the desktop vision and backlog, not a record of
implemented or verified changes. Track priorities in [TODO.md](TODO.md#desktop-vision).

Current setup: Debian, Sway, foot, fuzzel, nvim, yazi and Zen browser. Already
close to the desired setup. The goal is a desktop that looks and feels like one
deliberate system while staying minimal.

## Decisions so far

- **Launcher:** stay on fuzzel. It runs only when called, and it is small, maintained and fits the minimal goal.
- **Lua config managers** (zerb, Dotty, rdm, sm): skip. They add a layer for little gain on one machine. Git plus GNU Stow or chezmoi is enough.
- **Custom shells:** not needed now. Future reference:
  - **Quickshell:** Qt, configured in QML plus JavaScript.
  - **AGS:** GTK, configured in TypeScript plus JSX and CSS, running on GJS (not Node). Astal libraries cover battery, network, audio, tray, apps and notifications. Use Sway IPC directly for workspaces if no ready-made module is available.
  - Both are persistent processes; 50–200 MB is a planning ballpark, not a measured budget. Battery cost depends on avoiding polling.

## Fuzzel alternatives (reference)

Options recorded with the plan; recheck project status if reconsidering the launcher.

- **Minimal:** wmenu (plainer than fuzzel), bemenu, tofi (development quiet).
- **Heavier:** rofi (now native Wayland), walker (Go), anyrun (Rust).
- **Stalled:** wofi.

## 1. Fix notification history

Implementation update (2026-10-15): Mako remains the configured daemon. A
themed history viewer is installed on **Super+Shift+N** and in the launcher as
**Notifications**, with capacity for 100 history entries. The user confirmed
the viewer opens in the desktop. See [usage](notifications.md). Super+N is already
used for notes; the original restore-binding suggestion below is reference only.

- [ ] Find which daemon is running:

  ```sh
  pgrep -a 'mako|dunst|swaync'
  ```

- [ ] If it is mako, bind restore in the Sway config:

  ```text
  bindsym $mod+n exec makoctl restore
  ```

  To list history:

  ```sh
  makoctl history
  ```

- [ ] Consider SwayNotificationCenter (swaync) if a real panel with history and do not disturb is wanted.

## 2. Single-source color palette

- [ ] Choose tinted theming (base16 templates) or wallust (palette from wallpaper).
- [ ] Apply the palette to foot, nvim, fuzzel, mako, swaybar, shell, lock screen and Zen.
- [ ] Consider building a custom palette instead of a popular one; prioritize readable contrast.

## 3. Typography and geometry

- [ ] Use one font family everywhere, in a Nerd Font variant for icons.
- [ ] Use one corner radius, border width and gap size across Sway, fuzzel and mako where supported.
- [ ] Try **SwayFX**, a Sway fork with compatible configuration, for rounded corners, blur, shadows and dimming inactive windows.

## 4. Catch apps that ignore the theme

- [ ] Set a GTK theme for file pickers, plus qt6ct or Kvantum for Qt apps.
- [ ] Match cursor and icon themes.
- [ ] Set the system dark mode preference through the portal.
- [ ] Restyle Zen browser.
- [ ] Optionally use darkman for scheduled light and dark switching.

## 5. Consistent feel (keys)

- [ ] Use hjkl movement everywhere: Sway, nvim, yazi, zsh vi mode and the browser (Tridactyl or Vimium).
- [ ] Use the same close or quit key across tools.
- [ ] Add Sway modes that mirror the nvim leader key.

## 6. Polish start to finish

- [ ] Theme the boot loader and configure quiet boot.
- [ ] Consider greetd with tuigreet and the matching palette. Reconcile this with the existing TODO for disk-passphrase-only boot and automatic Sway login before choosing the login flow.
- [ ] Theme the lock screen with swaylock-effects or gtklock, including palette, font and clock.
- [ ] Add mako rules with distinct urgency colors and mute noisy apps.

## 7. Sway features to try

- [ ] **Marks:** label windows, jump to them and swap by mark.
- [ ] **Custom binding modes:** launch mode, system mode and leader-key style.
- [ ] **Bind by physical key position** (`--to-code`) so shortcuts survive layout switches.
- [ ] **Touchpad gestures** for workspace switching, plus a lid switch binding.
- [ ] **Keyboard layouts:** US plus US International with a toggle, or a compose key, for accents in Spanish, French, Italian and Portuguese.
- [ ] **for_window rules:** float dialogs, assign apps to workspaces and inhibit idle on fullscreen.
- [ ] **Scratchpad plus sticky** for a dropdown terminal, notes or dictionary.
- [ ] **Smart gaps and smart borders,** tabbed default layout and workspace back and forth.
- [ ] **Swaybar extras:** hide until modifier, tray and JSON status with click events.
- [ ] **IPC:** a small TypeScript or Go daemon for auto-tiling, workspace renaming or per-window layouts; follow the repository's language conventions when implementing.

## 8. Extras

- [ ] Switch the system locale to Spanish or Italian for passive input.
- [ ] Add a bar module showing course progress, daily language input or a Latin word of the day.

## Priority order

1. Notification history (quick win).
2. Single-source palette.
3. SwayFX.
4. Matching movement keys.
5. Sway features from section 7, then edges from section 6.
