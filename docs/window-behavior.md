# Window behavior

[Documentation index](README.md)

How Sway, the popup guard and the terminal shortcut handle fullscreen, floating
windows and touchpad gestures.

## Fullscreen and new windows

Sway keeps `popup_during_fullscreen smart`: a native dialog that names a
fullscreen parent appears above it and stays visible when the parent is
focused. An independent window, such as a new Foot terminal, does not. The
Yazi portal picker is such a window, so its wrapper leaves fullscreen before
launching Foot; see [portals](portals.md#desired-fullscreen-behavior).

`sway-popup-guard.service` starts with the Sway session and reacts to IPC
events; it never polls. When a window opens on a workspace with a fullscreen
window, including windows Sway already reports visible, the guard exits that
fullscreen, floats and centers the new window, and focuses it. The repo's own
terminal popups (Foot, the Yazi picker, scratch terminals) are also resized to
up to 800 by 550 logical pixels and 65% of the workspace; native dialogs keep
the size they ask for, since a forced frame left a small dialog inside a large
empty border. It restores fullscreen after the
tracked windows close or leave the workspace. Moving a floating window into a
fullscreen workspace, delayed visibility after focus or title events, and
floating windows already hidden at startup are handled the same way.

Explicit fullscreen has priority. Super+F enters fullscreen even when older
floating windows exist, clears the guard's pending restoration for that
workspace, and later focus or title events on those floaters do not undo it.
Restoration state is in memory: if the service is stopped mid-interruption,
press Super+F to restore fullscreen yourself. Stop the guard for the session
with `systemctl --user stop sway-popup-guard.service`.

With a single tiled background window this looks like fullscreen throughout.
With several tiled windows, the original layout shows during the interruption.

## Terminal shortcut during fullscreen

Super+Enter runs `terminal`. Inside fullscreen it exits the focused fullscreen
container, or global fullscreen, and launches Foot with a
`fullscreen-terminal-<parent>-<mode>` app ID that a Sway rule floats and centers
at 800 by 550. The guard recognizes that ID and restores the parent's
fullscreen when the terminal closes or moves to another workspace or output.
Outside fullscreen the shortcut opens a normal tiled terminal.

## Floating Foot opacity

In glass mode, floating Foot windows use at least 60% background opacity and
tiled ones drop the tint. Solid mode stays opaque. This covers upload and save
pickers and windows toggled with Super+Shift+Space; Foot server sessions are
not covered. The guard sends Foot's OSC 11 sequence when a window changes
between floating and tiled.

After installing an updated guard:

```sh
systemctl --user restart sway-popup-guard.service
```

Or log out and back in. Restarting the browser alone does not reload the guard.

## Fullscreen limits

Sway hides the wallpaper behind native fullscreen. Foot has no image-background
setting, so a fullscreen window looks solid. Preserving the pattern behind a
fullscreen window would need compositor support.

## Touchpad gestures and status panel

Super+I toggles the status panel. Four-finger swipe down shows it and swipe up
dismisses it. The panel is a Fuzzel window with on-demand keyboard focus and
stays open when another window takes focus back, so it behaves like a
scratchpad toggle without being one; only its process group is closed. Escape
closes it while it has focus.

Three-finger swipes focus the window in that direction. Four-finger left and
right switch to the next or previous workspace on the current output.
Four-finger pinch outward and inward send Ctrl+= and Ctrl+- through wtype;
apps decide what those do. Tap to click, disable-while-typing and clickfinger
are on; natural scrolling is off. Physical gesture verification is pending.

## Verification

Offline tree-snapshot tests cover reveal, delayed visibility, moves and focus,
startup recovery, multiple popups, restoration, native visible dialogs, hidden
workspaces, explicit fullscreen priority and the opacity transitions with real
PTY delivery. An isolated headless Sway 1.10.1 session with real Foot windows
confirmed popup size, floating, focus and fullscreen restoration, and that a
GTK dialog with a transient parent stays visible above its fullscreen parent.

Pending on hardware: mouse-dragging a floating window across physical outputs
(the user has seen this work; a two-output headless IPC test did not reproduce
that path), mixed DPI, rotation, hotplug and global fullscreen.

Sources: [Sway transient visibility logic](https://github.com/swaywm/sway/blob/1.10.1/sway/tree/view.c)
and [popup condition](https://github.com/swaywm/sway/blob/1.10.1/sway/tree/container.c).
