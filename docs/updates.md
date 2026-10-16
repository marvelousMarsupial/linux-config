# Automatic Debian updates

[Documentation index](README.md)

Use Debian's `unattended-upgrades` and the packaged `apt-daily.timer` and
`apt-daily-upgrade.timer`. No custom cron job or update service is needed.

The project policy refreshes package lists and attempts upgrades daily, and
cleans cached package downloads weekly. It permits the installed Debian release's
stable, security and stable-updates repositories. This includes ordinary stable
maintenance fixes, not just security fixes. Codename matching prevents an
automatic upgrade to the next Debian release. Third-party repositories,
backports and separately installed binaries are outside this policy.

Automatic reboots and automatic dependency/kernel removal are disabled. Package
installation scripts may still restart affected services. Reboot at a convenient
time after kernel updates; running applications may also need restarting to use
updated libraries. Review unused packages manually with `sudo apt autoremove`.

## Setup and verification

`./scripts/setup` installs both repo APT policy files, backs up replaced versions
and enables Debian's update timers. No separate manual policy edits are required
on a fresh installation. Both policy files matched the development VM in the
2026-10-08 read-only check after the user ran setup.

An isolated `apt-config` check confirmed the intended origins and the
periodic, reboot and removal settings. A real privileged upgrade dry run,
current timer health and delivery after a future update still need live
verification; see [TODO](TODO.md#security).

OpenSnitch may prompt for outgoing update traffic.

## Optional manual maintenance

Normal fresh installation uses `./scripts/setup`. The commands below are for
manual policy maintenance or troubleshooting from the repository root in an
unrestricted terminal. The config-only installer does not change `/etc` or services.

```sh
sudo apt-get update
sudo apt-get install unattended-upgrades

# Preserve existing versions before installing the project policy.
backup_dir="/etc/apt/linux-config-backup-$(date +%Y%m%d-%H%M%S)"
sudo install -d -m 0700 "$backup_dir"
for name in 20auto-upgrades 90linux-config-unattended-upgrades; do
    if sudo test -e "/etc/apt/apt.conf.d/$name"; then
        sudo cp -a "/etc/apt/apt.conf.d/$name" "$backup_dir/"
    fi
done
sudo install -m 0644 system/apt/apt.conf.d/20auto-upgrades /etc/apt/apt.conf.d/
sudo install -m 0644 system/apt/apt.conf.d/90linux-config-unattended-upgrades /etc/apt/apt.conf.d/

# Inspect the merged settings, including any later local overrides.
apt-config dump | sed -n '/APT::Periodic/p; /Unattended-Upgrade/p'
sudo unattended-upgrade --dry-run --debug
sudo systemctl enable --now apt-daily.timer apt-daily-upgrade.timer
systemctl list-timers --all apt-daily.timer apt-daily-upgrade.timer
```

The dry run must show only the intended Debian origins. Investigate any errors
before relying on automation. Existing package exclusions and APT holds remain
in effect. Later files in `apt.conf.d` can override this policy.

The packaged timers use randomized delays and persistent scheduling to catch
missed runs after downtime. Their services may defer on battery power; inspect
`systemctl cat apt-daily.service apt-daily-upgrade.service` on the machine.
A powered-off or sleeping laptop does not install updates in the background.

## Check and maintain

```sh
systemctl --failed
journalctl -u apt-daily.service -u apt-daily-upgrade.service --since '7 days ago'
sudo tail -n 50 /var/log/unattended-upgrades/unattended-upgrades.log
```

Review failures periodically; enabled timers alone do not prove updates succeeded.
For an immediate real run after reviewing the dry run, use
`sudo unattended-upgrade --verbose`. This installs eligible updates.

Periodically review the remaining updates with `sudo apt-get update` and
`apt list --upgradable`, then apply them interactively with `sudo apt upgrade`.
Update Yazi, Neovim, plugins and other separately installed tools through
their own installation methods. They are not covered by APT automation.
Firefox itself is a Debian package; its configuration package follows the pin
below.

## Pinned upstream releases

Yazi, the Nerd Font and the firefoxconfig package are installed from the release tags and SHA-256
digests recorded in `releases.toml`, so a setup run installs exactly what was
reviewed. Check for newer upstream releases every few weeks:

```sh
./scripts/releases check
```

A non-zero exit means something is behind. Read the upstream release notes,
then record the new tag and the digests GitHub publishes for the same assets:

```sh
./scripts/releases bump
```

Review the diff and commit it. Setup only installs these applications when
they are missing, so an existing machine updates them by removing the old
copy or through the application's own updater. Neovim is pinned separately in
`scripts/setup-editor`.

References: [Debian periodic updates](https://wiki.debian.org/PeriodicUpdates),
[Debian unattended-upgrades manual](https://manpages.debian.org/trixie/unattended-upgrades/unattended-upgrades.8.en.html).

## Desktop notifications

The Sway session target starts `update-notify.timer`, which checks APT history
30 seconds after session start and every two minutes. Completed transactions
containing package upgrades produce one desktop notification; failed/incomplete
transactions and package-list refreshes do not. This covers manual APT upgrades
as well as unattended upgrades. A reboot reminder is included when Debian's
`/var/run/reboot-required` flag exists; absence of that flag is not proof that
all running programs use current libraries.

The first run records existing history without announcing old updates. Later
updates installed while logged out are announced at the next session, provided
they are still in the current or previous compressed APT history log. Notification
failures are retried on the next check. Your user must be able to read those logs.
The notifier does not report failed update attempts; check the system logs above.

After installing user files, activate notifications in an existing Sway session:

```sh
systemctl --user daemon-reload
systemctl --user start update-notify.service
systemctl --user start update-notify.timer
```

New sessions start the timer automatically. Check it with
`systemctl --user status update-notify.timer` and
`journalctl --user -u update-notify.service`. Mako displays the notifications.
