# Optional encrypted home backups

[Documentation index](README.md)

Each machine can opt into Restic backups independently. Setup copies the helper
and user units but never enables the timer, creates a repository or uploads data.
Use `./scripts/setup --backups` to include the Restic package on a new machine,
or `sudo apt-get install restic` on an existing installation.

Restic encrypts repository contents with the repository password. Store a recovery
copy of that password and target credentials separately, for example in your safe.
The local password file allows unattended backups; it is readable by your account.
No passwords or targets belong in this Git repository.

## Enable on one machine

Run `./scripts/install` first, then:

```sh
install -d -m 700 ~/.config/restic-home
install -m 600 examples/restic-home.env ~/.config/restic-home/env
${EDITOR:-nano} ~/.config/restic-home/env
(umask 077; read -r -s -p 'Backup password: ' backup_password; echo; printf '%s\n' "$backup_password" > ~/.config/restic-home/password)
```

Set `RESTIC_REPOSITORY` to your actual target. Use an absolute path outside home
for a local disk, or a Restic backend URL for remote storage. Set up backend
credentials in the private env file when needed. This file uses Bash syntax and
is trusted code. For SFTP, arrange noninteractive SSH authentication and verify
the server's host key first. User services do not inherit your interactive shell's
environment; use explicit credential paths/settings rather than relying on it.

For a **new** repository, initialize it explicitly:

```sh
restic-home init
```

For an existing repository, use its password and skip init. Test before scheduling:

```sh
restic-home backup --dry-run
restic-home backup
restic-home snapshots
restic-home check
systemctl --user daemon-reload
systemctl --user enable --now restic-home.timer
```

The timer runs daily with up to 30 minutes of jitter. A missed run is caught up
when the user manager next starts. It runs while your user manager is active;
it does not wake a powered-off machine or automatically enable login lingering.
It continues independently of Sway while the user manager remains running.
An unavailable target produces a failed run, visible in the journal; the next
daily run tries again. No service failure notifications are added.

## Contents and recovery

Backups cover the current user's entire home, including hidden files, SSH keys,
configs, documents and KeePassXC databases. `~/.cache`, the Restic configuration
directory and the password file are excluded. Symlinks are saved as links;
external targets are not followed. Other mounted directories under home may be
included. Additional per-run exclusions can be passed with `--exclude`.
Running applications can change files during backup; this is not an atomic
filesystem snapshot. System directories and other users are outside its scope.

No automatic retention/deletion is configured. Monitor repository capacity and
choose a retention policy separately. Never run prune/forget without reviewing it.

Test recovery into an empty directory, using a snapshot ID from `snapshots`:

```sh
mkdir -m 700 ~/restic-restore-test
restic-home restore SNAPSHOT_ID --target ~/restic-restore-test
```

Files appear under the original path inside that target (for example
`~/restic-restore-test/home/august/`). Check a restored file before relying on the
backup. Use an explicit snapshot ID to avoid selecting another machine's latest
snapshot in a shared repository. The helper forwards restore arguments to Restic;
choose the target carefully because existing files can be overwritten.

```sh
systemctl --user list-timers restic-home.timer
systemctl --user status restic-home.service
journalctl --user -u restic-home.service
systemctl --user disable --now restic-home.timer
```

Disabling the timer leaves existing backups intact. An already running backup
can be stopped separately with `systemctl --user stop restic-home.service`.
Config installation preserves the locally created env/password files because
no templates for those paths are installed.

References: [Restic backup options](https://restic.readthedocs.io/en/stable/040_backup.html)
and [restore behavior](https://restic.readthedocs.io/en/stable/050_restore.html).
