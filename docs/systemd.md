# Systemd desktop session

[Documentation index](README.md)

Sway starts `~/.local/bin/sway-session` once from its config. The helper checks
that its IPC connection works, imports an explicit allowlist of desktop
environment variables into D-Bus and systemd, and starts `sway-session.target`.
A second active session is left untouched; this setup supports one Sway
session per user.

The target starts kanshi, Mako, the popup guard, the battery monitor, swayidle,
udiskie, the OpenSnitch UI, `update-notify.timer` and the one-shot `vpn-off`
that disconnects every VPN at login. No enable command is
needed. Mako starts only through systemd to avoid duplicate notification
daemons. Startup resets failures only for the services this integration owns,
preserving unrelated failure evidence.

`sway-session-monitor.service` keeps a Sway IPC subscription open. When Sway
exits or the connection is lost, the monitor exits, and `BindsTo=` stops the
session target with it. A later login imports the new compositor environment
and starts fresh services. Reloading the Sway config does not restart the
session. `graphical-session.target` orders the jobs; ordering does not prove
that display discovery has finished.

The Foot socket override removes a startup ordering cycle between desktop
services and the basic user session.

## Install and inspect

```sh
./scripts/install --dry-run
./scripts/install
systemctl --user daemon-reload
```

Log out of Sway and back in after changing units. Adding a `BindsTo=`
dependency to an already active target can stop the target during
`daemon-reload`; if that happens in a running session, restore it with
`systemctl --user start sway-session.target`. Sway reload does not restart
services and does not run new `exec` lines.

Inside Sway:

```sh
systemctl --user status sway-session.target sway-session-monitor.service kanshi.service
systemctl --user status battery-monitor swayidle udiskie sway-popup-guard opensnitch-ui
journalctl --user -b -u sway-session-monitor.service -u kanshi.service
systemctl --user --failed
```

From another TTY after leaving Sway, confirm the units are inactive. Log in
again and confirm they activate with working monitor profiles. Kanshi restarts
on failure with a two-second delay and a bounded rate; manual stops are
respected. A machine without a battery lets the battery monitor exit
successfully without restart loops.

## Locking and power

`swayidle.service` runs one idle watcher with the session:

```sh
swayidle -w timeout 1800 'swaylock -f' before-sleep 'swaylock -f'
```

This locks after 30 minutes without input and before suspend. `-w` waits for
swaylock to finish before releasing the sleep inhibitor, within logind's
`InhibitDelayMaxSec`. Fullscreen windows do not inhibit idle on their own;
application-requested inhibitors such as video playback are respected. Super+Escape
and the power menu lock manually. Test idle locking, manual locking and
suspend/resume on the real machine before enabling any auto-login plan.

`system/systemd/logind.conf.d/lid.conf` is optional lid policy; see
[installation](installation.md#local-and-system-configuration).

## Sway logs

Sessions started from tty1 send Sway output to the journal:

```sh
journalctl --user -b -t sway
```

## Graphical authentication

Full setup installs [mate-polkit](https://packages.debian.org/trixie/mate-polkit).
Sway starts `/usr/libexec/polkit-mate-authentication-agent-1` directly at
login so the agent belongs to the desktop login session. Launching it from the
user manager can register it against a different session, which is why the
older `sway-polkit-agent.service` was retired; the config installer removes it
with a backup. Test from a Sway terminal:

```sh
pkexec --disable-internal-agent /usr/bin/id
```

Expect a graphical dialog; after authenticating it prints `uid=0(root)`. The
user confirmed this works. Config-only installation does not install the
package. GNOME SessionManager warnings in the log are not a Polkit failure.

## Timers

`sysreport.timer` runs the health report daily with `Persistent=true`; see
[system health](sysreport.md). `update-notify.timer` and `restic-home.timer`
are described in [updates](updates.md) and [backups](backups.md).

References: [systemd dependencies](https://github.com/systemd/systemd/blob/main/man/systemd.unit.xml),
[Sway IPC client](https://github.com/swaywm/sway/blob/master/swaymsg/main.c),
[swayidle manual](https://github.com/swaywm/swayidle/blob/master/swayidle.1.scd),
[mate-polkit registration](https://github.com/mate-desktop/mate-polkit/blob/v1.26.1/src/main.c).
