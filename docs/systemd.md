# Systemd desktop session

[Documentation index](README.md)

Sway starts `~/.local/bin/sway-session` once from its config. The helper checks
that its IPC connection works, imports an explicit desktop environment allowlist,
and starts `sway-session.target`. A second active session is left untouched;
this setup supports one Sway session per user.

The target starts kanshi, Mako, the popup guard, battery monitor, swayidle, udiskie and `update-notify.timer`. No separate enable command is needed.
Mako starts only through systemd to avoid duplicate notification daemons. The notification timer is described in [updates](updates.md).
Kanshi restarts on failure with a two-second delay and a bounded restart rate.
Manual service stops are respected. Startup resets failures only for kanshi, the session monitor and the three
desktop jobs owned by this integration, preserving unrelated failure evidence.

`sway-session-monitor.service` keeps a Sway IPC subscription open. When Sway exits
or its connection is lost, the monitor exits. `BindsTo=` with `After=` stops the
session target; kanshi stops through `PartOf=`. Stopping the target also stops
the monitor. A later login imports the new compositor environment and starts
fresh services. Reloading the Sway config does not restart the session.

`graphical-session.target` is the shared desktop coordination point. Dependencies
order jobs; they do not prove that displays have finished discovery or guarantee
that a particular kanshi version has no protocol races.

## Install and inspect

```sh
./scripts/install --dry-run
./scripts/install
systemctl --user daemon-reload
```

Adding a new `BindsTo=` dependency to an already active target can stop the
target during `daemon-reload` if the new dependency is inactive. For this
migration, the simplest application path is to log out and back in after the
reload.
If the target stopped in an existing Sway session with its environment already
imported, restore it with `systemctl --user start sway-session.target`.

Log out of Sway and log back in after changing units. These commands copy user
files and reload definitions; they do not change `/etc` or system services.

Inside Sway:

```sh
systemctl --user status sway-session.target sway-session-monitor.service kanshi.service
journalctl --user -b -u sway-session-monitor.service -u kanshi.service
systemctl --user --failed
```

From another TTY after leaving Sway, confirm all three units are inactive. Log in
again and confirm they activate with working monitor profiles. Reload Sway and
check that the service PIDs stay the same. Test suspend and resume on the actual
machine and confirm the screen locks first.

## Power and other services

`system/systemd/logind.conf.d/lid.conf` is optional system configuration: suspend
on lid close, including external power, and ignore lid close while docked.
Review it for the machine before copying it into `/etc/systemd/logind.conf.d/`.
Apply it during a planned reboot; restarting logind can disrupt sessions.

`swayidle.service` starts one idle watcher with the Sway session:

```sh
swayidle -w timeout 1800 'swaylock -f' before-sleep 'swaylock -f'
```

This locks after 30 minutes without input and before suspend. `-w` waits for
`swaylock -f` to finish locking before releasing the sleep delay inhibitor,
subject to logind's `InhibitDelayMaxSec` limit. Fullscreen windows no longer
automatically inhibit idle; application-requested inhibitors (for example video
playback) are still respected. Manual Super+Escape and the power menu still lock.

For the initial migration, log out and back in to replace the old Sway-started
processes without duplicates. Later unit changes need `daemon-reload` and a
service restart. Sway reload does not restart these services.
Test idle locking with a fullscreen terminal and no input for 30 minutes, then
unlock with the account password. Also test manual locking and suspend/resume.
Confirm the screen is locked on resume. Do these checks before relying on the
auto-login plan; auto-login itself has not been enabled.

See the [upstream swayidle manual](https://github.com/swaywm/swayidle/blob/master/swayidle.1.scd)
for lock readiness and suspend-delay behavior.

The three desktop jobs stop with `sway-session.target` and restart on failure
with a two-second delay and bounded retries. A battery-free VM lets the battery
monitor exit successfully; it does not repeatedly restart. udiskie retains
automounting and notifications without a tray. Inspect them with
`systemctl --user status battery-monitor swayidle udiskie` and
`journalctl --user -b -u battery-monitor -u swayidle -u udiskie`.
Live activation of this migration still needs logout/relogin.
The automated checks cover script behavior. The IPC disconnect test uses the
installed client and a disposable socket; it explicitly skips when the sandbox
prohibits local sockets. This does not replace a live desktop integration test.

References: [systemd dependencies](https://github.com/systemd/systemd/blob/main/man/systemd.unit.xml)
and [Sway IPC client](https://github.com/swaywm/sway/blob/master/swaymsg/main.c).

The Foot socket override removes a startup ordering cycle between desktop
services and the basic user session. Shutdown still stops the socket.

## Sway logs

New tty1 sessions send Sway output to the journal.
After logging out and back in, read it with:

```sh
journalctl --user -b -t sway
```

The current session cannot recover output already sent to tty1.

## Graphical authentication

Full setup installs [Debian's mate-polkit package](https://packages.debian.org/trixie/mate-polkit).
Sway starts `/usr/libexec/polkit-mate-authentication-agent-1` directly at login.
The agent determines its login session from its process, so launching it through
systemd's user manager can register it for a different session from desktop apps.
A running process alone does not verify working authentication prompts.
See [upstream registration code](https://github.com/mate-desktop/mate-polkit/blob/v1.26.1/src/main.c).
Existing authorization policy stays in effect.

The previous `sway-polkit-agent.service` integration is retired and backed up by
config installation. To migrate an existing session:

```sh
systemctl --user stop sway-polkit-agent.service
./scripts/install
systemctl --user daemon-reload
```

Log out of Sway and back in; a config reload does not run new `exec` commands.
Then test in a Sway terminal:

```sh
pkexec --disable-internal-agent /usr/bin/id
```

Expect a graphical dialog. Cancel once, then repeat and authenticate; success
prints `uid=0(root)`. Test again after another login. This restricted session
cannot launch or verify the live desktop agent. Config-only installation does
not install the package. GNOME SessionManager warnings are not themselves a
Polkit registration failure; that optional step follows agent registration.

On 2026-10-08 the user confirmed this authentication test works after applying
the direct Sway startup fix.

## System health timer

`sysreport.timer` runs `sysreport.service` with `OnCalendar=daily` and
`Persistent=true`. It writes a local HTML file as the user; no server is needed.
Setup enables it, including an on-disk link when no user bus is available. The
live schedule and automatic run still need verification. See [system health](sysreport.md).

The session launcher resets only the failed subset of its managed services,
avoiding errors for services that have not been loaded yet.
