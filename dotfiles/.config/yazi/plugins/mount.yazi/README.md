# mount.yazi (vendored)

This is the upstream [mount.yazi][upstream] plugin, copied verbatim into
the dotfiles tree so it ships embedded inside the `forge` binary and
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
and `eject` — all already pulled in by the desktop role's
`udiskie` / `udisks2` packages, so no extra install is needed.

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
cargo build --release -p forge   # re-embed
```

If the API ever changes incompatibly the embed-test
`embedded_dotfiles_has_core_files` will keep catching missing files;
runtime breakage would surface as a yazi error popup when pressing `M`.

## License

MIT — see `LICENSE` in this directory. Original copyright is held by the
yazi-rs maintainers.

[upstream]: https://github.com/yazi-rs/plugins/tree/main/mount.yazi
