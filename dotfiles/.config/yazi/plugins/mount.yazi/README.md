# mount.yazi (vendored)

This is the upstream [mount.yazi][upstream] plugin, copied verbatim into
the dotfiles tree so it can be copied by the installer and
"just works" on a fresh machine — no `ya pkg add ...` step required.

## What it does

Opens an interactive picker inside yazi listing every block device the
system can see (mounted *and* unmounted). Inside the picker:

| Key                | Action                                |
|--------------------|---------------------------------------|
| `j` / `k` / `↑↓`   | Move cursor                           |
| `m`                | Mount the selected partition          |
| `u`                | Unmount the selected partition        |
| `e`                | Eject the selected device             |
| `l` / `→` / `Enter`| Jump into the mount point and quit    |
| `q` / `Esc`        | Close the picker                      |

Backed by `udisksctl` (mount/unmount/eject), `lsblk` (enumerate devices),
and `eject`. The package list includes udiskie and its storage dependencies.
Verify these commands are available when testing removable media.

## Where it's bound

In `dotfiles/.config/yazi/keymap.toml`:

```toml
{ on = "M", run = "plugin mount", desc = "Mount picker (USB / drives)" }
```

So inside yazi, hit shift-M to open the picker.

## Updating

The upstream is at [`yazi-rs/plugins`][upstream]. To refresh:

```bash
curl -fsSL https://raw.githubusercontent.com/yazi-rs/plugins/main/mount.yazi/main.lua \
    > dotfiles/.config/yazi/plugins/mount.yazi/main.lua
./scripts/install              # copy the updated plugin
```

Review upstream changes before replacing the vendored file. Test the picker
interactively after updating; syntax checks cannot certify Yazi API compatibility.

## License

MIT — see `LICENSE` in this directory. Original copyright is held by the
yazi-rs maintainers.

[upstream]: https://github.com/yazi-rs/plugins/tree/main/mount.yazi
