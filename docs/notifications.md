# Notifications

[Documentation index](README.md)

Open **Notifications** from the app launcher or press **Super+Shift+N**.
The themed fuzzel viewer shows active notifications followed by recent history.
Type to search across app, title and body; press Enter to read the selected
message. Escape returns from the detail view, and Escape again closes the list.
Reopen the viewer to refresh its snapshot. Notification markup is shown as text;
the viewer does not invoke actions or dismiss messages.

Mako keeps up to 100 expired/dismissed notifications in memory. History ends
when Mako exits or the session ends; it is not a permanent log. Alias hints and
status OSD messages are excluded. Super+N still opens notes.

After installing the files, activate the config from the desktop:

```sh
makoctl reload
swaymsg reload
```

Offline tests cover opening a message, control-character sanitization, empty
history, cancellation and a failed session connection. Theme render checks pass.
The user confirmed the viewer opens in the desktop. The current sandbox cannot
reach the live compositor/session bus to inspect its running state.

The wrapper queries Mako through `busctl` and uses existing `jq` and `fuzzel`
packages. This avoids depending on the changing `makoctl` output format; see
[Mako release notes](https://github.com/emersion/mako/releases).
