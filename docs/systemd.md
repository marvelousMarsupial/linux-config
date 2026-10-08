# Systemd desktop session

[Documentation index](README.md)

Sway starts `~/.local/bin/sway-session` once from its config. The helper checks
that its IPC connection works, imports an explicit desktop environment allowlist,
and starts `sway-session.target`. A second active session is left untouched;
this setup supports one Sway session per user.

The target starts kanshi, Mako and `update-notify.timer`. No separate enable command is needed.
Mako starts only through systemd to avoid duplicate notification daemons. The notification timer is described in [updates](updates.md).
Kanshi restarts on failure with a two-second delay and a bounded restart rate.
Manual service stops are respected. Startup resets failures only for the two
services owned by this integration, preserving unrelated failure evidence.

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

Sway starts one idle watcher at login:

```sh
swayidle -w timeout 1800 'swaylock -f' before-sleep 'swaylock -f'
```

This locks after 30 minutes without input and before suspend. `-w` waits for
`swaylock -f` to finish locking before releasing the sleep delay inhibitor,
subject to logind's `InhibitDelayMaxSec` limit. Fullscreen windows no longer
automatically inhibit idle; application-requested inhibitors (for example video
playback) are still respected. Manual Super+Escape and the power menu still lock.

Log out and back in to apply watcher changes. Sway reload does not replace an
already running `exec` watcher. On the VM, the installed config matches the repo,
but sandbox restrictions prevent activating or verifying the live watcher.
Test idle locking with a fullscreen terminal and no input for 30 minutes, then
unlock with the account password. Also test manual locking and suspend/resume.
Confirm the screen is locked on resume. Do these checks before relying on the
auto-login plan; auto-login itself has not been enabled.

See the [upstream swayidle manual](https://github.com/swaywm/swayidle/blob/master/swayidle.1.scd)
for lock readiness and suspend-delay behavior.

On the current VM, the session target, IPC monitor, kanshi and notification
timer are active. Logout/relogin and suspend locking have not been exercised.
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
