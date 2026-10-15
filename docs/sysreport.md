# System health

[Documentation index](README.md)

The installed Bash command `~/.local/bin/sysreport` has no checkout dependency.
Open `~/.local/share/sysreport/latest.html` in your browser.
Reports are private (directory 0700, HTML 0600); the newest 14 dated reports remain
alongside the atomic latest copy. No server, JavaScript, or external assets are used.

Refreshing the browser reads the saved file; it does not collect new data.
Running `sysreport`, manually or through its daily timer, replaces `latest.html`
with a new snapshot. A later browser refresh can therefore show changed values.
Check the report's `Generated` timestamp to identify the snapshot. Dated
`report-*.html` files retain their captured values until retention removes them.

The overview has eight status tiles: backups, updates, services, resources, disk
health, journal, network, and OpenSnitch. Green means confirmed success, red a
failure or resource alert, and amber attention or unknown data. Update checks must
have a successful service completion within 48 hours to turn green; pending
packages, stale metadata, and reboot markers keep them amber. Backup success must
be confirmed within 26 hours. Command output and issue lists are collapsed into
native HTML details panels, which work without JavaScript.

Run `sysreport --demo` to write `~/.local/share/sysreport/demo.html` with eight
green sample tiles. It does not collect system data or replace `latest.html`.

Setup reloads user units and enables `sysreport.timer` when a user bus is present.
Without a user bus, setup creates the enablement link for the next login.
For a config-only install, run:

```sh
systemctl --user daemon-reload
systemctl --user enable --now sysreport.timer
~/.local/bin/sysreport
systemctl --user list-timers sysreport.timer
```

The installer records its commit, modified-checkout flag, completion date and scope
in `~/.local/state/linux-config/install-info`. A successful full setup replaces the
config-install receipt with a setup receipt. Failed setup never records full success.
No previous installation date is invented when the receipt is absent.

## OpenSnitch

In the main OpenSnitch window, click the gear beside the top-left menu button.
On the Database tab, choose File and select `~/.local/share/opensnitch.db` using
the full home path. Save, quit the UI completely, and restart its user service.
Changing the preference does not move an already-running in-memory database.

The report reads the configured database read-only. An empty database stays amber.
It lists rules created in the last 24 hours, denied destinations and connection
counts by process. It does not show browser page URLs or page contents.

Setup installs a root-owned systemd override for the daemon with
`-ui-socket unix:///run/user/UID/opensnitch/osui.sock`, using the installing user's
UID. Existing daemon preferences and rules are preserved; a replaced project
override is backed up. Setup enables the daemon and restarts it when the override
changes. This workstation configuration targets one desktop user; running setup
as another user reassigns the daemon's UI connection.

Sway starts the repository's `opensnitch-ui.service`, which creates its private
runtime directory and launches the themed UI on the matching socket. The
no-argument launcher uses the same runtime socket. Log out/in after setup to
activate the session changes. Starting `/usr/bin/opensnitch-ui` directly without
the socket argument can instead use `/tmp/osui.sock` and leave collection empty.
Persistent database storage remains an optional UI preference as described above.

## Journal

The tile counts warning/error entries over 24 hours, including repeated messages;
the details group identical messages with counts. Old entries remain until they
age out, even after a fix. Missing journal access is shown separately in the tile.
The report does not filter away compatibility warnings or clear logs.

## Verification

Offline checks cover escaping, permissions, retention, section failures, green
service status and demo isolation. ShellCheck, the live timer schedule and a
real receipt from a full install still need checking on the machine; OpenSnitch
collection was confirmed by the user. See [TODO](TODO.md#security).

Required tools: Bash, coreutils, util-linux (flock, lsblk), procps, systemd,
iproute2, apt and jq. sqlite3 enables optional OpenSnitch file-database queries.
smartmontools enables optional SMART health inspection, with no disk self-tests.

No machine-specific report configuration is required. OpenSnitch preferences are
read, never changed. SQLite is opened read-only with query_only and zero busy wait;
short reads may briefly hold shared locks, but never exclusive write locks. Unknown
schemas or unavailable sources produce section errors and summary warnings.

APT counts use existing metadata; the report never refreshes lists or installs
packages. CPU percentages are lifetime averages. SMART needs access to the physical
device on many machines: do not add the user to the disk group (raw disk access).
If needed, separately review a root-owned helper restricted to fixed device health
queries and an exact sudoers command; this implementation does not install one or
invoke sudo. System journal access can be granted via systemd-journal membership;
this does not grant direct access to private unattended-upgrade log files.

Backup snapshot time is evidence of a snapshot, not proof of an exit-0 backup:
Restic can create a snapshot on a partial run. Service state and retained journal
success events provide separate evidence. No repository size scan is run.
