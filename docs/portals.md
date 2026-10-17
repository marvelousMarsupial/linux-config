# Sway portals and the Yazi file picker

[Documentation index](README.md)

The repo routes file dialogs to termfilechooser (Yazi in Foot), screenshots and
screen sharing to wlr, and other supported dialogs to GTK. The Sway-specific
routing applies when `XDG_CURRENT_DESKTOP=sway`. GTK is the file-dialog
fallback if termfilechooser is absent. `sway-session` imports the compositor
environment into D-Bus and systemd; no extra portal startup command belongs in
Sway.

## Install the backends

Full `./scripts/setup` installs the backends and user configs. Debian 13
provides the wlr and GTK backends but no termfilechooser package, so setup
builds it from the pinned upstream commit, which corresponds to v1.4.3. The
manual equivalent, from a host terminal with network access and sudo:

```sh
sudo apt install xdg-desktop-portal xdg-desktop-portal-gtk xdg-desktop-portal-wlr build-essential pkgconf git meson ninja-build libinih-dev libsystemd-dev scdoc
mkdir -p ~/.local/src
cd ~/.local/src
git clone https://github.com/hunkyburrito/xdg-desktop-portal-termfilechooser.git
cd xdg-desktop-portal-termfilechooser
git checkout --detach cb8cec0b30aa6e2cc36b1b5cb34d6dc9a1face29
meson setup build --prefix=/usr --libexecdir=lib
meson compile -C build
sudo meson install -C build
```

The `/usr` prefix puts the backend metadata where Debian's portal broker
expects it. This is a source install, not an APT package; updates need an
intentional rebuild. Then run `./scripts/install` from the repo root.

## Activate and use

Fully log out of Sway and back in, then start Firefox through the repo
launcher (`~/.local/bin/firefox`, Super+B). The launcher sets `GTK_USE_PORTAL=1`,
which Mozilla's default `widget.use-xdg-desktop-portal.file-picker=2`
(automatic) honors. If that preference was customized, set it to **1** in
`about:config`.

Yazi opens in a separate floating Foot window. Enter selects a file; Space
selects several before confirming a multiple-file request. For saving, the
backend creates an instructional placeholder: rename or move it in Yazi to the
wanted name and location, then select it. For directory requests, selecting a
directory returns it and quitting with `q` accepts the current directory;
close the Foot window to cancel. For file requests, `q` without a selection
cancels. A cancelled request leaves an empty result file so the backend reads
it without error; a failed terminal launch still fails the request. Filenames
containing newlines are unsupported by the line-based wrapper protocol.

## Desired fullscreen behavior

Confirmed by the user and regressed before; preserve it when changing portal,
Sway or window rules.

- Opening an upload or save picker from fullscreen Firefox first leaves Sway
  fullscreen, then shows Yazi in a centered, focused floating Foot window.
- Selecting a file or cancelling closes the picker and restores the original
  fullscreen, unless another window has since been made fullscreen.
- The picker wrapper handles the fullscreen transition itself before launching
  Foot, independently of the popup guard. Floating alone can leave the picker
  hidden behind the fullscreen browser.

## Verify on the live desktop

- In Firefox, test Ctrl+O and a web upload with one and several files.
- Repeat an upload while Firefox is fullscreen; check restoration after selection
  and after cancel.
- Test Save As to a new filename, cancel, and overwrite of a disposable file.
- Test directory upload and cancellation.
- Try filenames with spaces, quotes, dollar signs and non-ASCII characters.
- Test screen sharing and confirm the chosen Sway output is captured.
- If dialogs fail, inspect the portal services:

  ```sh
  systemctl --user status xdg-desktop-portal.service xdg-desktop-portal-wlr.service xdg-desktop-portal-termfilechooser.service
  journalctl --user -b -u xdg-desktop-portal.service -u xdg-desktop-portal-termfilechooser.service
  ```

To return to GTK dialogs, set `org.freedesktop.impl.portal.FileChooser=gtk` in
`~/.config/xdg-desktop-portal/sway-portals.conf`, then log out and back in and
restart Firefox.

Status: the user confirmed a single-file upload through Yazi and the
fullscreen exit and reveal, on the earlier Zen setup. Offline tests cover
literal filenames, single and multiple selection, cancellation, directory
fallback cleanup and the Firefox launcher environment. The Firefox picker
still needs a live check. Save and overwrite, directory requests, screen sharing
and fullscreen restoration after selection and cancel still need live checks.

## Sources

- [Portal routing specification](https://flatpak.github.io/xdg-desktop-portal/docs/portals.conf.html)
- [wlr integration](https://github.com/emersion/xdg-desktop-portal-wlr)
- [termfilechooser](https://github.com/hunkyburrito/xdg-desktop-portal-termfilechooser)
- [Wrapper protocol](https://github.com/hunkyburrito/xdg-desktop-portal-termfilechooser/blob/main/xdg-desktop-portal-termfilechooser.5.scd)
- [Mozilla portal preferences](https://github.com/mozilla-firefox/firefox/blob/main/modules/libpref/init/StaticPrefList.yaml)
