# Zen browser setup

[Documentation index](README.md)

`scripts/install` copies browser defaults and extension policies into the local
Zen installation at `~/.local/zen`. They take effect on the next complete browser
restart, including the first launch of a new profile. `scripts/setup` installs
Zen before copying these files. No browser restart is performed by setup.

The tracked policy installs uBlock Origin, Dark Reader, Surfingkeys and
KeePassXC-Browser from Mozilla Add-ons. Zen downloads the signed extensions on
startup, so an internet connection is needed. `normal_installed` leaves each
extension available to disable or remove in `about:addons`; these are not locked
extensions. Check `about:policies` for active policies and any errors.

Application defaults skip Zen's welcome screen and request dark browser and
website colors before a profile exists. The existing `zen-theme-sync` integration
applies desktop palette CSS once a default profile exists. Dark Reader handles
sites that need recoloring. Its settings remain in the browser profile.

Zen's redundant title bar/window controls are disabled by application defaults
and profile sync. Use Super+Q to close the window in Sway.

The glass sidebar uses 8px blur; expanded search uses 14px. Restart Zen after
changing the CSS. Upload/save pickers use a floating Foot window with 60%
background opacity; see [window behavior](window-behavior.md#floating-foot-opacity).

KeePassXC's native messaging manifest is installed by the same config installer.
Open KeePassXC, enable browser integration, unlock your database, and connect
the extension to it. Database approval and pairing remain interactive.

## Surfingkeys settings

The source is `dotfiles/.surfingkeys.js`, copied to `~/.surfingkeys.js` by the
installer. The config uses rounded Dracula colors, the installed CaskaydiaCove
font, smooth scrolling and a larger Vim editor. `jj` acts as Escape in page
inputs, visual mode, the omnibar and the Vim editor's insert mode. Normal-mode
`j` keeps its default scroll behavior.

Surfingkeys for Firefox 1.17.11 does not implement the newer `<native>` config
loader described on upstream's main branch. Copy the installed file's contents
into Surfingkeys' settings editor (`;e`, enable advanced mode if needed), then
save. Repeat after changing the tracked snippets. Installing the extension or
copying the file does not automatically import extension settings.

Do not put API keys or other credentials in the tracked snippets.

### Try a config change

Run `wl-copy < ~/.surfingkeys.js`, open the Surfingkeys extension settings, enable
advanced mode, replace the snippets with the clipboard contents, and save.
Use a terminal inside Sway for clipboard access. If this VM's shell has no
`WAYLAND_DISPLAY`, use `WAYLAND_DISPLAY=wayland-1 wl-copy < ~/.surfingkeys.js`.
Reload a regular HTTPS page. Extensions cannot run on protected browser pages
such as `about:addons` or Mozilla Add-ons pages.

1. Click a text input, type a word, then press `jj`. It should leave insert mode
   without adding `jj`; `j` should then scroll the page. Also test typing a single
   `j` and pausing to check whether sequence handling suits your typing.
2. Press `t` to open the omnibar. Check the rounded dark panel and purple selected
   row, then press `jj` to close it. Test `/` and `jj` for the find bar as well;
   cancellation behavior can differ between front-end modes.
3. Press `f` and check the rounded yellow hints. Escape cancels hints.
4. In a multiline text field, use the editor shortcut (`Ctrl-i` by default),
   press `i`, type, then `jj`. It should return to Vim Normal mode while keeping
   the editor open. `:q!` discards the test edit and closes the editor.

The user confirmed `jj` works in the live browser. The other checks still need
browser testing; syntax checks do not verify key handling or layout.

## Updates

Edit the repo files and rerun `scripts/install` after replacing/updating Zen's
application directory, since upstream archives can replace its `defaults` and
`distribution` directories. The config installer backs up files it replaces;
it does not copy or replace browser profiles, bookmarks, logins or extension
storage. Browser policy may display a management notice in Zen's settings.

References: [Mozilla extension policies](https://firefox-admin-docs.mozilla.org/reference/policies/extensionsettings/),
[Mozilla application defaults](https://support.mozilla.org/en-US/kb/customizing-firefox-using-autoconfig),
[Surfingkeys](https://github.com/brookhong/Surfingkeys).
