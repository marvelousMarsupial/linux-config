# Sway portals and the Yazi file picker

The repo routes file dialogs to termfilechooser (Yazi in Foot), screenshots and
screen sharing to wlr, and other supported dialogs to GTK. The Sway-specific
configuration applies when `XDG_CURRENT_DESKTOP=sway`. GTK is the file-dialog
fallback if termfilechooser is absent, not a recovery mechanism for a crashing
backend. Existing `sway-session` imports the compositor environment into D-Bus
and systemd; no extra portal startup command belongs in Sway.

## Install the backends

The full `./scripts/setup` command installs these backends and user configs.
The manual equivalent follows for portal-only maintenance.

Debian 13 provides wlr and GTK, but this VM's APT index has no termfilechooser
package. Build the latter from the reviewed upstream release, not a download
executed directly as a shell script. Run from a normal host terminal with network
access and sudo:

```sh
sudo apt install xdg-desktop-portal xdg-desktop-portal-gtk xdg-desktop-portal-wlr build-essential git meson ninja-build libinih-dev libsystemd-dev scdoc
mkdir -p ~/.local/src
cd ~/.local/src
git clone https://github.com/hunkyburrito/xdg-desktop-portal-termfilechooser.git
cd xdg-desktop-portal-termfilechooser
git checkout --detach cb8cec0b30aa6e2cc36b1b5cb34d6dc9a1face29
git rev-parse HEAD
meson setup build --prefix=/usr --libexecdir=lib
meson compile -C build
sudo meson install -C build
```

The pinned commit corresponds to v1.4.3. The `/usr` prefix puts the backend
metadata where Debian's portal broker expects it. This is a local source install,
not an APT-managed package; later updates require an intentional rebuild.
Install the repo user files with `./scripts/install` from the repo root.

## Activate and use

Fully log out of Sway and log back in, then restart Zen through the repo launcher.
The launcher sets Wayland and enables portal file dialogs for Mozilla's default
`widget.use-xdg-desktop-portal.file-picker=2` (automatic). If that preference was
customized, set it to **1** in Zen's `about:config` to explicitly enable the portal.
This setup does not rewrite browser profiles or change other applications'
environment variables.

Yazi opens inside a separate Foot window. Select a file with Enter; use Space to
select several before confirming a multiple-file request. Its floating window uses
60% background opacity in glass mode and stays opaque in solid mode. For saving, the backend
creates an instructional placeholder: rename/move it in Yazi to the desired name
and location, then select it. Verify save and overwrite behavior before relying
on it. For directory requests, selecting a directory returns it; quitting with
`q` accepts the current directory, matching upstream. Close the Foot window to
cancel a directory request instead. For file requests, `q` without a selection
cancels. Filename newlines are unsupported by the line-based wrapper protocol.
Cancelled file requests leave an empty result file so the backend can read the
result without a missing-file error. A failed terminal launch still fails the
request; it is not treated as cancellation.

## Desired fullscreen behavior

This is the intended behavior, confirmed by the user on 2026-10-08. It has
regressed in the past; preserve it when changing portal, Sway or window rules.

- Opening an upload or download/save picker from fullscreen Zen must first
  leave Sway fullscreen, then show Yazi in a centered, focused floating Foot window.
- Selecting a file or cancelling closes the picker and restores the original
  fullscreen mode, unless another window has since been made fullscreen.
- The picker wrapper handles the fullscreen transition before launching Foot;
  keep this independent of the background popup handler. Floating alone can leave
  the picker hidden behind the fullscreen browser.

## Verify on the live desktop

- In Zen, test Ctrl+O and a web upload with one and multiple files.
- Repeat an upload while Zen is fullscreen: fullscreen must exit before the
  floating picker appears. Check fullscreen restoration after selection and cancel.
- Test Save As to a new filename, cancel, and overwrite of a disposable file.
- Test directory upload and cancellation; verify the selected directory.
- Try filenames containing spaces, quotes, dollar signs and non-ASCII characters.
- Test screen sharing and confirm the chosen Sway output is captured.
- Inspect `systemctl --user status xdg-desktop-portal.service
  xdg-desktop-portal-wlr.service xdg-desktop-portal-termfilechooser.service`
  and `journalctl --user -b -u xdg-desktop-portal.service
  -u xdg-desktop-portal-termfilechooser.service` if dialogs fail.

To return to GTK, set `org.freedesktop.impl.portal.FileChooser=gtk` in
`~/.config/xdg-desktop-portal/sway-portals.conf`, then log out/in and restart Zen.

## Sources

- [Portal routing specification](https://flatpak.github.io/xdg-desktop-portal/docs/portals.conf.html)
- [wlr integration](https://github.com/emersion/xdg-desktop-portal-wlr)
- [termfilechooser installation and configuration](https://github.com/hunkyburrito/xdg-desktop-portal-termfilechooser)
- [Wrapper protocol](https://github.com/hunkyburrito/xdg-desktop-portal-termfilechooser/blob/main/xdg-desktop-portal-termfilechooser.5.scd)
- [Mozilla portal preferences](https://github.com/mozilla-firefox/firefox/blob/main/modules/libpref/init/StaticPrefList.yaml)

## Status — 2026-10-08

User routing, wrapper, Zen launcher and termfilechooser backend are installed in
the VM. The user confirmed a working Zen upload through Yazi and then confirmed
that opening the picker from fullscreen exits fullscreen and reveals the floating
picker. Automated checks pass; live save/overwrite, directory requests, screen
sharing and fullscreen restoration after both selection and cancellation still
need separate verification.
