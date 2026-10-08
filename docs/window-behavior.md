# Floating dialogs and fullscreen

The current Sway configuration retains `popup_during_fullscreen smart`.
Native dialogs that identify a fullscreen parent can appear above it, and stay
visible when the background window is focused. Floating windows already stack
above ordinary tiled windows on the same workspace.

The `sway-popup-guard` user service is configured to start with the Sway session.
Current live service health cannot be checked from this sandbox. New windows
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
The Yazi portal picker is an independent Foot window. Its wrapper now exits
fullscreen before launching Foot; the user confirmed that an upload picker is
visible with this behavior. See [desired picker behavior](portals.md#desired-fullscreen-behavior). Independent
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
so the terminal appears solid despite that alpha. Terminals opened with Super+Return
over a fullscreen window float with at least 60% background opacity for readability;
solid mode remains opaque. The popup guard also adjusts all standalone Foot windows
when they switch between floating and tiled: floating gets at least 60%, tiled
returns to the current theme alpha. This includes Yazi upload/save pickers and
custom terminal app IDs. Foot's supported OSC 11 sequence changes the background
without fading text. The existing Sway event subscription drives updates; no new
service or polling is added. Foot server/client sessions are not covered.
After updating the popup guard, restart `sway-popup-guard.service` in the user
session or log out/in to load the new behavior.
The Zen-only local-image
workaround does not cover Foot. Preserving the dot wallpaper behind native
fullscreen Foot remains unresolved; it requires compositor support or a different
fullscreen/terminal approach. The default fullscreen binding remains unchanged.

## Always reveal newly opened windows

New windows should leave fullscreen and appear floating, including windows Sway
already reports visible. The old visible-window exception was removed at the
user's request. Existing floating windows do not undo a later explicit fullscreen
action merely because focus or title changes.

The user reports successfully dragging a floating window over fullscreen from
another physical monitor. A two-output headless test of IPC moves did not reproduce
that exact behavior. Mouse-drag and physical-output validation remain pending;
do not infer that the user's observed path is impossible from the IPC move test.

## Delayed visibility and moved windows

The guard rechecks pending new windows after focus/title events, including focus
returning to the fullscreen parent, and handles floating windows moved into a
fullscreen workspace. Startup also recovers already-hidden floating windows.
New visible dialogs also trigger the fullscreen-exit behavior. Duplicate
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

Both newly opened tiled and floating windows trigger reveal, even if the initial
IPC snapshot reports them visible.

## Restore after a shortcut terminal closes or leaves

The terminal app ID identifies the original fullscreen container and mode.
The popup guard tracks that relationship even though fullscreen was exited
before the terminal appeared. Closing the terminal or moving it to another
workspace/output restores the original window's fullscreen, after the last
tracked popup leaves. Moving within the same workspace does not restore it.
Super+F still overrides automatic restoration. Reload Sway and restart the
popup guard after installing this revision.
