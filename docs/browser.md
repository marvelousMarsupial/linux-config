# Firefox setup

[Documentation index](README.md)

The browser is Debian's `firefox-esr`, installed from `packages.txt`. Its
configuration comes from a separate repository,
[firefoxconfig](https://github.com/marvelousMarsupial/firefoxconfig), which
builds a `.deb` that drops policies, preferences, `userChrome.css` and
autoconfig scripts into `/usr/lib/firefox-esr`. Every profile gets the same
setup; nothing is written into profiles by this repository.

`scripts/setup` installs the package version pinned in `releases.toml`,
verifies its SHA-256, runs `dpkg -i`, and clears each profile's startup cache
so changed scripts load. A rerun of setup upgrades the package when the pin
moves. Close Firefox before running setup. The package depends on
`firefox-esr`, so Firefox itself still updates through APT.

Extensions, search engines, the hidden UI and the dark theme are all defined
in firefoxconfig; read its README for the manual steps after the first start,
including pasting its Surfingkeys settings.

## What this repository adds

- `~/.local/bin/firefox` runs `firefox-esr` with `GTK_USE_PORTAL=1`, which makes
  Mozilla's automatic file-picker setting use the desktop portal, so upload and
  save dialogs open Yazi. Super+B uses it. See [portals](portals.md).
- A user copy of `firefox-esr.desktop` points the launcher and link handlers at
  the same wrapper.
- The KeePassXC native messaging manifest in `~/.mozilla/native-messaging-hosts/`.
  See [passwords](passwords.md).

## Updating the pin

Tag a release in firefoxconfig, then:

```sh
./scripts/releases check
./scripts/releases bump firefoxconfig
./scripts/setup
```

The asset name in `releases.toml` contains `{version}`, which expands to the
tag without its leading `v`. When the package moves to another host, add
`url = "https://host/path"` to the entry; setup then downloads
`<url>/<asset>` instead of the GitHub release, and the digest check stays.
