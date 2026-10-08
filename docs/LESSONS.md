# Project lessons

Updated: 2026-10-08. Add lessons when a change exposes a reusable rule; keep outstanding work in [TODO.md](TODO.md).

## Configuration and UX

- The setup script targets a fresh system and configures the desired state without manual settings edits: keep IPython's scientific stack and set KeePassXC/OpenSnitch dark through installation and app launchers. Do not add cleanup for software found only on the development VM or trim requested dependencies just because the system should stay lightweight.
- Dark mode is a fresh-install requirement across the desktop, KeePassXC and Zen. GTK settings alone do not cover Qt apps. Keep the setup lightweight: seed independent app themes explicitly, export GTK settings before Sway starts, and import them for activated apps; avoid extra theme plugins unless an actual app needs them. See [dark defaults](appearance.md#dark-mode-is-the-default).
- Preserve the confirmed Yazi portal behavior: leave Sway fullscreen before opening the centered floating picker, then restore fullscreen when it closes. Floating alone can hide the picker behind Zen. Handle this in the picker wrapper before Foot launches, independently of the popup service; see [desired fullscreen behavior](portals.md#desired-fullscreen-behavior). This behavior has regressed before.
- Put application-specific labels in the application's launcher, not shared Fuzzel settings. A global “Search applications” placeholder also appeared in the read-only info popup, password picker and power menu.
- Keep one palette source and generate app colors from templates. Editing generated files alone loses changes on the next theme apply.
- Preserve local monitor layouts, plugin lockfiles and selected colors when copying configs to an existing machine.
- Keep the selected palette consistent across apps. Earlier theme application stopped partway through and left different apps using different palettes.
- Do not send undocumented reload signals. Foot's SIGUSR1 hook could terminate a terminal and interrupt theme application. New terminals read their colors; existing ones can be reopened.
- Improve spacing, typography and contrast before adding visual software. The current refresh uses only existing programs and configuration.
- Remove an integration completely: executable, service, shortcut, aliases, status controls, theme targets, package requirement and docs. Removing the main script alone leaves broken UI.
- Minimal default tools and optional development tooling are separate choices. Keep compilers, Node and language-server installation explicit.
- Yazi needs `file` for type detection. Valid TOML does not establish that preview dependencies or runtime file opening work.

- Starting a desktop target does not connect it to compositor exit. Track the IPC connection and bind the target to that service; ordering alone does not establish application readiness.

## Script correctness

- Treat cancelled or failed menus as no action, even if they print output. Match selections against known choices before acting.
- Never delete a saved network profile merely because authentication failed. It can contain user-managed addressing and other settings.
- Keep passwords out of command-line arguments and notifications. The Wi-Fi helper now supplies prompted input through stdin; its real authentication flow still needs testing.
- Use unique, private temporary files for notes and screenshots. Second-resolution timestamps can collide and overwrite data.
- Quote paths and use argument lists. Validate options and path boundaries separately; quoting does not prevent an argument from being interpreted as an option.
- Validate interface names before constructing privileged config paths, including names obtained by numeric selection. The old VPN helper is now retired.
- Back up before replacing files; use atomic replacement per file. Neither the installer nor theme application is a transaction across every destination.
- Validate backup destinations before creating directories or changing their permissions; destination-file checks alone leave the backup path open.
- Treat malformed compositor responses as errors rather than an empty window list.
- Parse NetworkManager standard connection types and escaped saved-profile names; display labels are not reliable protocol values.
- Restrict destination directory symlinks so installation cannot escape the chosen home. Preserve existing leaf symlinks in backups without writing through them.
- Exclude Python bytecode and caches from installation, including files created by test imports.
- Bound optional data queries and distinguish missing tools from empty results. UI helpers should degrade without hiding important installation failures.

## Security

- Prefer no automatic clipboard history when reliable secret exclusion is unavailable. Removing startup capture must also stop the existing watcher; reloading Sway alone does not stop it.
- Clipboard clearing is not history deletion. Installed pass restores/clears clipboard content later, while cliphist can retain the earlier password copy. The installed wl-clipboard cannot reliably classify all sensitive selections.
- A suspend lock does not provide inactivity locking. Avoid blanket fullscreen idle inhibition; distinguish it from application-requested inhibitors. Use swayidle `-w` with swaylock `-f` so the suspend hook waits for lock readiness (within logind’s delay limit). Changes to an `exec` watcher require logout/login; Sway reload leaves the old process running.
- Parse runtime environment files as data, not shell code. A private directory helps, but it is not a reason to execute a file unnecessarily.
- Verify downloaded executable assets against pinned expected digests before extraction or execution. TLS and checksum pinning address different parts of the trust chain; a checksum is not a publisher signature.
- Pin bootstrap code too. A plugin lockfile does not help if the manager itself is first loaded from a moving branch.
- A helper’s version probe executes it; version matching does not verify integrity. Disable an unverified integration at its load paths as well as removing automatic downloads.
- Download hooks and optional package managers remain supply-chain inputs. Pinning the editor and manager does not audit all plugins or transitive binaries.
- Use private backup directories and explicit file modes. Do not assume the user's umask excludes group access.
- Bind temporary development servers to loopback by default. Sharing the current directory over all interfaces should be explicit.
- Keep preview servers local and disable unnecessary editor modeline/project configuration loading.
- Removing packages from the future-install list does not stop or uninstall existing services. Retiring container tooling must not remove the disposable VM's guest agent or remote-access path.

## Verification and reporting

- Give each document one purpose: current guides, dated audit evidence, open tasks or reusable lessons. Replace outdated claims instead of appending corrections indefinitely.

- Save tests must follow the editor’s policy: compare whole prose files exactly, allow code formatting, and use a deterministic external formatter to verify the save hook and disk output even when optional tools are missing.
- Use real application parsers as well as syntax checks. Yazi theme keys had drifted even though the TOML parsed correctly.
- Test failure paths: cancellation, missing binaries, malformed input, symlinks, failed downloads and partial installs.
- Use temporary homes and command stubs for network, power and privileged actions. Passing those tests does not prove the real action works.
- Sandbox networking/process views cannot certify the host's listening services. Enabled service links are not proof that a service is running.
- An empty firewall file is not proof that no rules are loaded. Read the effective ruleset before changing networking, especially over a remote connection.
- For isolated APT configuration checks, set `Dir::Etc::parts` through an `APT_CONFIG` bootstrap file. Passing it with `apt-config -o` changes the displayed setting after the normal configuration has already loaded.
- Cached APT candidates and enabled unattended-update timers do not prove a system is current. Refresh metadata and inspect service results.
- Secret scans need a stated scope. No matches across common patterns and reachable Git history is useful evidence, not a guarantee of no secrets.
- Keep audit reports dated. Track unresolved findings in TODO rather than presenting all tested code as completely secure.

References and detailed evidence: [security audit](security-audit.md), [script review](scripts.md), [verification](testing.md), and [appearance notes](appearance.md).
