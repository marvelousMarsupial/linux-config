# Floating dialogs and fullscreen

The current Sway configuration retains `popup_during_fullscreen smart`.
Native dialogs that identify a fullscreen parent can appear above it, and stay
visible when the background window is focused. Floating windows already stack
above ordinary tiled windows on the same workspace.

The tested `sway-popup-guard` user service is restored and active. New windows
opening during fullscreen are centered and floated (up to 800×550 and 65% of the
workspace). The helper temporarily disables native fullscreen to reveal them;
it restores fullscreen when the intervening windows close or leave the workspace.
With a single tiled background window this can look like fullscreen throughout.
Clicking the tiled background leaves floating windows above it. The existing
layout can become visible if other tiled windows share the workspace.

This is the behavior the user observed working and requested back. The service
is wanted by `sway-session.target`, uses IPC events rather than polling, and stops
with the session. `systemctl --user stop sway-popup-guard.service` disables it for
the current session. Restoration state is in memory; if the service is stopped
while fullscreen is interrupted, use Super+F to restore fullscreen manually.

## Verified behavior on Sway 1.10.1

In an isolated headless Sway session, a GTK dialog with a transient parent
remained visible while the parent stayed fullscreen. Explicitly focusing the
parent left the dialog visible. A separate Foot terminal, forced floating at
500×300, was reported invisible while that same parent remained fullscreen.
This distinguishes parented dialogs from independent application windows.
Download/file dialogs should use the parented-dialog path, but actual Zen and
portal dialogs still require application-specific verification. Independent
terminals above true fullscreen are not resolved by the stock popup setting;
the restored helper reveals them by temporarily disabling native fullscreen.
No maximized-layout replacement or compositor patch was installed.

## Zen fullscreen appearance

Sway omits the desktop wallpaper behind native fullscreen. The custom glass CSS
paints a local `glass-wallpaper.png` behind Zen chrome only in that state, keeping
web pages opaque. `wallpaper-texture` updates this local copy; dot/grid patterns
repeat and other textures use cover. Restart Zen to load the CSS change.
Disabling glass removes the managed import, including this fullscreen treatment.

An isolated Zen test verified the fullscreen root attribute, local wallpaper
background, and 8% tint. A screenshot confirmed the dot pattern. Hardware,
multiple outputs, global fullscreen, and real application dialogs remain pending.

Source: [installed-version Sway transient visibility logic](https://github.com/swaywm/sway/blob/1.10.1/sway/tree/view.c)
and [popup condition](https://github.com/swaywm/sway/blob/1.10.1/sway/tree/container.c).

## Foot fullscreen limitation

Foot 1.21.0 has background alpha but no wallpaper-image setting in its installed
manual. Its live alpha remains 0.08. In native fullscreen Sway hides the wallpaper,
so the terminal appears solid despite that alpha. The Zen-only local-image
workaround does not cover Foot. Preserving the dot wallpaper behind native
fullscreen Foot remains unresolved; it requires compositor support or a different
fullscreen/terminal approach. The default fullscreen binding remains unchanged.

## Preserve popups Sway already displays

The helper now skips the fullscreen-exit workaround when a new window is already
reported visible above fullscreen. Already-visible native dialogs are left alone. Tests with the helper active confirmed a transient GTK
dialog stays above a genuinely fullscreen parent after focusing the parent;
independent newly opened terminals still pass the reveal/restore tests.

The user reports successfully dragging a floating window over fullscreen from
another physical monitor. A two-output headless test of IPC moves did not reproduce
that exact behavior. Mouse-drag and physical-output validation remain pending;
do not infer that the user's observed path is impossible from the IPC move test.

## Delayed visibility and moved windows

The guard rechecks pending new windows after focus/title events, including focus
returning to the fullscreen parent, and handles floating windows moved into a
fullscreen workspace. Startup also recovers already-hidden floating windows.
Already-visible native dialogs retain native fullscreen behavior. Duplicate
floating/focus events do not resize a popup repeatedly. Commands racing with a
closing window are logged rather than terminating the service.

Offline tree-snapshot tests cover reveal, delayed visibility, move/focus, startup,
multiple popups, restoration, native visible dialogs, and hidden workspaces.
Live validation of this revision requires the desktop session; this sandbox
cannot connect to the compositor or user service manager.

## Explicit fullscreen has priority

Super+F is allowed to enter fullscreen even when older floating windows exist.
An explicit fullscreen-enable event clears the prior popup restoration session
and pending candidates on that workspace. Focus/title events on old floaters do
not immediately disable fullscreen again. New windows and floating windows
explicitly moved into the workspace still trigger reveal/restore as needed.
Regression tests cover both fullscreen entry and popups arriving afterward.

## Terminal shortcut during fullscreen

Super+Enter runs `terminal`, which queries the focused workspace, exits its
fullscreen container (or global fullscreen), and launches Foot with the
a `fullscreen-terminal-<parent>-<mode>` app ID. A Sway rule centers and floats that terminal at
800×550. This shortcut does not rely on the popup helper reacting after launch;
it leaves fullscreen before opening the window. Outside fullscreen, Foot opens
normally as a tiled terminal. Reload Sway after installing the binding and rule.

The popup helper only uses the already-visible exception for floating dialogs;
a newly opened tiled window still triggers reveal even if its initial IPC
snapshot reports it visible.

## Restore after a shortcut terminal closes or leaves

The terminal app ID identifies the original fullscreen container and mode.
The popup guard tracks that relationship even though fullscreen was exited
before the terminal appeared. Closing the terminal or moving it to another
workspace/output restores the original window's fullscreen, after the last
tracked popup leaves. Moving within the same workspace does not restore it.
Super+F still overrides automatic restoration. Reload Sway and restart the
popup guard after installing this revision.
