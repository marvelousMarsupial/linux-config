# CLAUDE.md

AI onboarding for this repository. Read this first.

## What this is

A single-binary Linux workstation provisioner. The `forge` CLI embeds the entire
dotfile tree, the role definitions, and the theme system at compile time; running
`forge setup <role>` on a fresh Debian box brings it from "stock install" to
"fully configured workstation" in one command, with no repo clone, no network
fetch (apart from explicit binary downloads), and no separate config lookup.

Owned by user **august**, lives at `/home/august/linux-config`.

```bash
~/linux-config/target/release/forge setup desktop
~/linux-config/target/release/forge status
~/linux-config/target/release/forge teardown
```

**Roles:** `server`, `desktop`. Two roles only — server is the base
(shell stack, editor, rootless-podman runtime, foot terminfo,
whisper runtime libs, **virt stack: incus + libvirt/qemu**) and
desktop extends it with the Sway compositor and the audio/dictation
stack (**plus virt-manager for graphical VM consoles**). Every role
can spin up dev pods via `forge dev` because the container packages
live in `server`, and every role can also run system containers
(incus) and full VMs (libvirt/qemu) — the unified `vm-menu` keybind
($mod+c) covers all three runtimes from a single fuzzel popup. There
is intentionally **no Rust toolchain** in any role — `forge` itself
is rebuilt inside a dev container and the binary is `podman cp`-ed
onto the host, keeping `~/.cargo` and `~/.rustup` off production
hosts.

## Repo layout

```
linux-config/
├── Cargo.toml              # workspace manifest
├── crates/
│   ├── common/             # forge_common: planner, executor, theme, state, ...
│   └── forge/              # binary crate; embeds dotfiles/ + config/ + themes/
├── dotfiles/               # mirrors $HOME — embedded into the binary
│   ├── .bashrc
│   ├── .zshrc
│   ├── .tmux.conf
│   ├── .config/{sway,foot,fuzzel,nvim,yazi,starship.toml,...}
│   └── .local/
│       ├── bin/            # scripts (notes-toggle, power-menu, wifi-menu, ...)
│       └── share/applications/
├── config/
│   ├── roles.toml          # role definitions (packages + dotfiles globs + post)
│   └── dev-container.toml  # single podman dev container image config
├── themes/
│   ├── config.toml         # template targets + scheme metadata
│   ├── schemes/*.yaml      # base16 color schemes
│   └── templates/*.j2      # tera templates rendered per-app
├── containers/             # base.Containerfile (single dev image,
│                           # embedded into the binary at compile time
│                           # so `forge dev build` works from any cwd)
├── legacy/                 # the previous Ansible+Stow+Python implementation
│                           # — reference only, do not extend
└── target/release/forge    # built binary
```

## How a `forge setup` works

1. **Parse** `config/roles.toml` (embedded as `&str`). Resolve the role's
   `extends` chain, deduplicating packages/dotfiles/downloads/post.
2. **Plan** (`forge_common::planner::build_plan`):
   - `AptUpdate` once if any package install follows
   - `InstallPackage` per dedup'd package
   - `Download` per binary (nvim, starship, yazi, fonts, whisper model)
   - `WriteFile` per dotfile that matches a glob. Mode 0o755 gets set
     on files starting with `#!` (shell scripts) **or** `\x7fELF`
     (vendored binaries — currently whisper-server / whisper-cli).
     Everything else lands at the default 0o644.
   - `PostCommand` per `post` entry
3. **Execute** (`forge_common::executor::execute`): every action is idempotent.
   `is_installed`, content-equality checks in `write_atomic`, `dest.exists()`
   for downloads, and the optional `check` field on post-commands all let
   reruns be cheap no-ops. `InstallPackage` also runs `apt-mark manual`
   on every package — including the already-installed path — so an
   `apt autoremove` later can't sweep a role-required package that
   originally landed as someone else's transitive dep. `Download` runs
   curl through `with_retry` (3 attempts, 2s + 4s exponential backoff)
   so a single transient failure (GitHub-Releases CDN 502, DNS hiccup,
   brief TLS handshake error) doesn't abort the entire `forge setup`
   plan — caught by tests in `executor::tests::with_retry_*`.
4. **State** is written to `~/.config/forge/state.json` so `forge status`
   and `forge teardown` know what role is live.

## Editing the dotfile tree

Every file under `dotfiles/` ships in the binary via `include_dir!`. Edit a
file, then `cargo build --release -p forge` to re-embed it. There is no
"sync from disk" mode — the binary is the source of truth at runtime.

`dotfiles/.local/bin/<script>` is the canonical home for new scripts. They
land at `~/.local/bin/<script>` and are made executable by the planner's
shebang detection. Sway's `$bin` is set to `/home/august/.local/bin`.

## Editing roles

`config/roles.toml`. A role can `extends` another (chain is resolved
base-first so leaf entries append/override). Each role declares:

- `packages` — apt package names
- `dotfiles` — globset patterns matched against the embedded tree
- `downloads` — `[[role.downloads]]` tables with `name/url/dest/extract/chmod`
- `post` — shell commands; bare string or `{ run, check }` for idempotency
- `teardown_post` — inverse commands run during `forge teardown`

Keep packages alphabetically grouped by purpose with a comment header. Keep
post-commands idempotent (always supply a `check` unless the command itself
is a true no-op on rerun).

## Theme system

`forge theme apply <name>` reads a base16 YAML scheme from `themes/schemes/`,
renders every Tera template in `themes/templates/` against the scheme, and
writes the output to the path declared in `themes/config.toml`. Targets can
declare a `hook` (run after write) or `restart` (process-name to kill/restart).

All scheme/template/config files are embedded in the binary just like dotfiles.

**Catppuccin-Mocha base03/base04 deliberately diverge from upstream
base16.** `themes/schemes/catppuccin-mocha.yaml` uses Overlay 2
(`#9399b2`) for `base03` and Overlay 1 (`#7f849c`) for `base04`,
not the upstream catppuccin/base16 export's surface1/surface2
(`#45475a`/`#585b70`). The upstream values are part of catppuccin's
*structural background* tier (intentionally low-contrast against
`base00 #1e1e2e`), but base03/base04 are what get used in practice
for comments, line numbers, and zsh-autosuggest text — at the
upstream values they render at ~1.5:1 against the bg, well below
WCAG's 3:1 minimum, visibly unreadable. Catppuccin's own style guide
(catppuccin/catppuccin docs/style-guide.md) explicitly assigns the
*text* tiers to Overlay 1 (subtle text / line numbers) and Overlay 2
(comments), so the divergence aligns the YAML with catppuccin's own
usage guidance rather than its byte-for-byte base16 export. Pinned
by `catppuccin_mocha_uses_overlay_for_text_tiers`. Don't "sync from
upstream" without re-reading that test's docstring — it's a
deliberate divergence, not a stale file.

## Dev container UX

The dev container is a daily-driver workspace, not a build sandbox.
A bunch of small fixes layer together to keep it ergonomic:

**`bat` symlink (`/usr/local/bin/bat -> /usr/bin/batcat`)** — Debian's
`bat` apt package installs as `batcat` due to a long-standing
filename clash with `bacula-console-bat` (Debian bug #930299, won't-
fix). Without the symlink the user's `cat = bat` zsh alias fails on
every invocation. The host's `server` role and the dev container's
`base.Containerfile` both create this symlink at the same path, so
host and container behave identically. Pinned by
`server_role_installs_bat_symlink` and
`base_containerfile_installs_bat_symlink`.

**Truecolor + OSC52 clipboard chain** — the visible "TUI graphics
are wrong" symptom inside dev containers was tmux stripping 24-bit
color escapes because `terminal-overrides ",xterm-256color:RGB"`
doesn't match `foot`. The fix has two halves:

  * `dotfiles/.tmux.conf` declares `terminal-features ",foot*:RGB"`
    and `",foot*:clipboard"` plus `set-clipboard on` (modern
    tmux ≥3.2 syntax — replaces the old regex `terminal-overrides`
    pattern). This tells tmux to pass truecolor and OSC52 escapes
    through to the outer foot terminal.
  * `containers/base.Containerfile` declares `ENV COLORTERM=truecolor`
    so TUI apps inside the container that detect truecolor via the
    `$COLORTERM` env (nvim, opencode, lazygit, btop, bat) see the
    flag — `podman exec -t` propagates `TERM` but NOT arbitrary env,
    so without this `ENV` line the container shell ends up with
    `COLORTERM` unset and apps quantise to 256-color.

Pinned by `tmux_conf_advertises_foot_truecolor_and_clipboard` and
`base_containerfile_sets_colorterm_truecolor`.

**OSC52 `wl-copy` shim inside the container** — host `wl-copy` talks
to the Wayland compositor, which doesn't exist inside a podman
container. The shim at `/usr/local/bin/wl-copy` (created by the
Containerfile via `printf > file && chmod`) emits the OSC52
sequence `ESC ] 52 ; c ; <base64> ESC \` so `clip` zsh alias and
tmux copy-pipe bindings that shell out to `wl-copy` work
unchanged. Paired with foot's default-on OSC52 support (since v1.4)
and the tmux `set-clipboard on` above. Paste is intentionally NOT
shimmed — OSC52 read is one-way-by-default in modern terminals
for security; use foot's native paste binding (Ctrl+Shift+V).
Pinned by `base_containerfile_installs_wl_copy_shim`.

**`forge` host-only stub** — `/usr/local/bin/forge` in the container
is a tiny `#!/bin/sh` that prints "forge is a host-only command"
and exits 127. Without it, typing `forge` in the container hits
zsh's command_not_found handler with a generic apt-index miss,
which is misleading (suggests forge is something installable).
Same pattern Gitpod uses for its `gp` host-integration command.
Pinned by `base_containerfile_installs_forge_stub`.

**Container DNS just works — but only after the host firewall
came off.** Earlier this session, `forge dev build` was failing
at the very first `apt-get update` with `Temporary failure
resolving 'deb.debian.org'` while the host could curl the same
URL fine. We chased it through three wrong fixes (pinning
`[containers] dns_servers` in containers.conf, passing `--dns=`
flags to `podman build`, considering `--network=host`) before
realising the root cause was the host's `inet forge_filter`
nftables table: pasta (rootless podman's network backend) sends
DNS UDP via host sockets, the replies came back into the input
chain, and `ct state established,related accept` was NOT
matching them on this kernel/pasta combo. With the firewall
rolled back to no-rules (see "Host firewall" above for the full
rationale), pasta's UDP path works again and builds need no
special flags.

Lessons worth remembering for next time:

  * `[containers] dns_servers` in containers.conf only applies
    to `podman run` — Buildah's build path ignores it
    (containers/podman issue #15290).
  * "Pin a public DNS fallback for resilience" is wrong on LANs
    that block outbound DNS to anything except the gateway
    (which is most LANs). The host's nameservers, whatever they
    are, are by definition reachable on the network the host is
    on; pinning public DNS overrides that with something WORSE.
  * Conntrack tracking of pasta's UDP queries isn't reliable on
    every kernel + pasta + nftables version combination. If a
    future host actually needs inbound filtering, restore via a
    per-host drop-in and explicitly test with `podman run
    --dns=<gateway> <image> getent hosts <name>` before
    declaring victory.

`~/.config/containers/containers.conf` still ships as a forge-
managed marker file but explicitly does NOT set `dns_servers`
(pinned by `dotfiles_ship_containers_conf_with_no_dns_override`).
The path is forge-owned so an ad-hoc edit can't squat it, but
podman falls through to its compiled defaults for both build
and run. `forge dev build`'s argv is the canonical podman
invocation (`podman build -f <containerfile> -t <tag>
<context>`) — no DNS shenanigans, no `--network=host`.

**TZ bind mounts** — `config/dev-container.toml`'s `default_volumes.bind`
mounts `/etc/localtime` and `/etc/timezone` from host into container,
read-only (`readonly = true` in TOML → podman's `:ro` mode suffix).
glibc's time-formatting code reads `/etc/localtime` directly, so a
bind mount is more reliable than `-e TZ=...` (which lots of glibc
code paths ignore). The `readonly` flag is non-negotiable: under
rootless podman + `--userns keep-id` the container UID maps onto
the invoking host user, so a writable mount would let a container
rewrite the host's zone configuration. The Containerfile installs
`tzdata` so the underlying zone files exist as a fallback when the
bind mount is absent (e.g. running `forge dev exec` under a
different runner). Pinned by `dev_container_provisions_tz_bind_mounts`
and `base_containerfile_installs_git_delta_and_tzdata`.

**lazygit + git-delta** — Both are workflow tools the dotfile
aliases reference (`lg` → lazygit; `delta` is a git-pager opt-in).
git-delta is in trixie main (apt). lazygit has no apt package on
trixie main, so the Containerfile pulls the upstream static
tarball at image-build time — same pattern as nvim/yazi/uv/fnm.
Pinned by `base_containerfile_installs_lazygit` and
`base_containerfile_installs_git_delta_and_tzdata`.

**`COPY dotfiles/.config/theme` is deliberately absent.** That tree
is templates + base16 schemes consumed by `forge theme apply` on
the host. Inside the container `forge` is intentionally a stub
(see above), so the templates never render — they'd just sit as
~100 KiB of inert .j2 + .yaml. The container consumes the
*rendered* outputs (`nvim/lua/plugins/colorscheme.lua`,
`yazi/theme.toml`, `tmux/colors.conf`) which DO get copied via
the explicit per-target dotfile COPYs, frozen at image-build time.
Pinned by `base_containerfile_does_not_copy_dead_theme_dir`. To
update the container's theme: run `forge theme apply <scheme>` on
the host, then `forge dev build` to rebuild the image, then
`podman volume rm <name>-home` so the next `forge dev up`
re-seeds `/home/dev` from the rebuilt image (the named volume's
first-run seed is what makes existing containers stale otherwise).

## Coding standards (load-bearing — enforced in review)

### 1. Rustdoc on every public item

Every `pub` item — modules, types, functions, traits, fields, enum variants —
gets a `///` doc comment that explains **what it is** and, if non-obvious,
**why it exists**. Every module gets a `//!` header that states its purpose
and any noteworthy design decisions (the existing modules in `crates/common`
are good models — see `planner.rs`, `executor.rs`, `atomic.rs`).

`cargo doc --no-deps --workspace` should build clean and read like a
self-contained reference. Write for someone joining the project cold.

```rust
//! Brief module purpose. Then a paragraph or two on design choices that
//! aren't obvious from reading the code.

/// What this function does. Trailing notes on edge cases or invariants.
///
/// # Errors
/// When/why this returns Err.
pub fn do_thing(x: u32) -> Result<()> { ... }
```

Private helpers don't need rustdoc but a one-line `//` comment is welcome
when the name doesn't already say it all. Keep comments short — explain
*why*, not *what*.

### 2. Tests for every behavior

Every module that does real work gets a `#[cfg(test)] mod tests` block.
The bar:

- New public functions get unit tests covering the happy path, the error
  path, and any explicit edge case the doc comment calls out.
- Idempotent operations get a "twice in a row" test (see
  `executor::tests::write_file_is_idempotent`).
- Anything that parses external input (TOML, YAML, glob patterns) gets a
  parse-success test, a parse-failure test, and a roundtrip if applicable.
- Embedded-asset invariants are tested in `embed.rs::tests` so a bad
  `roles.toml` or a missing template fails at `cargo test`, not at
  `forge setup` on a fresh machine.

`cargo test --workspace` must stay green. If you can't test something
(requires the live system, sudo, network) document it in the module header
and prefer extracting the testable core into a helper that takes the side
effect as an injected dependency — `DotfileSource` and `TemplateSource`
are the existing pattern.

### 3. Other expectations

- `cargo build --release -p forge` must succeed with zero warnings.
- Don't reach for new dependencies casually; the workspace already pulls in
  what's reasonable for a binary this size.
- Prefer extending `forge_common` over duplicating logic in `crates/forge`.
- Atomic writes go through `atomic::write_atomic`. Shell out via `run::checked`.

## Common tasks

**Add a new keybind script:** drop the script at `dotfiles/.local/bin/<name>`
(must start with `#!`), bind it in `dotfiles/.config/sway/config` via
`exec $bin/<name>`, ensure any apt packages it needs are in the desktop
role. Rebuild forge.

**Add a new role:** add a `[<name>]` section to `config/roles.toml`. The
`embed.rs::all_roles_resolve_and_plan` test will catch broken extends
chains and empty plans. Add the role to the `Role` enum in
`crates/common/src/role.rs`.

**Add a themed app:** add a Tera template under `themes/templates/`, add a
`[[targets]]` entry to `themes/config.toml` pointing at the output path
and (if needed) a hook or restart. The `embedded_themes_config_valid`
test enforces template existence.

**Investigate a setup failure:** `forge setup <role> --dry-run` shows the
plan; `forge setup <role> --diff` prints a unified diff per dotfile against
what's currently on disk.

**Spin up a clean test container or VM (`forge lxc` / `forge vm`):**
one-shot bootstrap for a fresh Debian trixie incus instance with the
forge binary pre-pushed. Replaces the manual "incus launch + override
IP + fix in-guest networkd + swap resolv.conf + useradd august + push
binary" sequence we used to do by hand for testing forge from zero.

```bash
forge lxc                  # default name forge-scratch, default IP 10.160.110.50
forge lxc test1            # custom name
forge lxc --no-shell test1 # bootstrap and exit (for scripting)
forge lxc --ip 10.160.110.99 test1
forge vm                   # KVM VM variant; default name forge-scratch-vm, IP .51
incus delete --force <name>  # cleanup when done
```

By default drops you into a shell as `august` (UID 1000, NOPASSWD
sudo, linger enabled) inside the guest with `./forge` ready to run.
The binary that gets pushed in is `~/linux-config/target/release/forge`
when present (so a `cargo build --release` followed by `forge lxc`
always tests the just-built version, even if `which forge` resolves
to a stale `~/.cargo/bin/forge`); falls back to the running
`current_exe()` otherwise. Implementation in
`crates/forge/src/commands/scratch.rs`.

The earlier draft of these commands ran `incus config device override
eth0 ipv4.address=…` followed by `incus restart` to register a
dnsmasq lease for the static IP — that turned out to be redundant
because the guest's `/etc/resolv.conf` is also swapped to public DNS
(1.1.1.1, 9.9.9.9), bypassing the gateway dnsmasq entirely. Removing
both fixed a load-bearing bug: VM restart on this host hangs for 6+
minutes per call (ACPI shutdown timing in the upstream image), which
made `forge vm` effectively unusable in the previous shape. Pinned by
the `default_name_and_ip_differ_between_lxc_and_vm` and
`vm_agent_timeout_exceeds_lxc` tests in the same module.

**Adjust the monitor layout (desktop role):** edit
`dotfiles/.config/kanshi/config`, not the sway config. Kanshi owns all
per-output mode/position/transform/scale; sway only ships the wildcard
wallpaper line. Profiles are matched by EDID description (`Make Model
Serial`) so connector-name shifts (DP-5 → DP-7 across dock reboots)
don't break anything. Two non-obvious gotchas, both burned us in
practice and are documented inline at the top of the kanshi config:

1. **Mode rate must be exact, including the fractional Hz.** `mode
   1920x1200@60Hz` failed silently against a panel that reports
   `@59.951Hz`, the profile aborted, and the laptop stayed dark on
   undock. Either pin the exact reported rate or omit `mode` entirely
   and let kanshi pick the preferred mode.
2. **Kanshi and `swaymsg output ... transform N` use opposite sign
   conventions for portrait rotations** on this kanshi version (1.7.0).
   `swaymsg ... transform 270` ⇔ kanshi `transform 90`, and vice
   versa. Verify visually after any rotation change.

After editing, rebuild forge (`cargo build --release -p forge`) and
rerun `forge setup desktop`. For a hot reload of the kanshi config
without going through forge, copy the file to `~/.config/kanshi/config`
and `systemctl --user restart kanshi.service` — kanshi is now managed
by a systemd user unit (`~/.config/systemd/user/kanshi.service`,
`WantedBy=sway-session.target`) rather than spawned directly by sway.
Lifecycle is owned by graphical-session.target so a sway exit cleanly
stops kanshi, and a kanshi crash mid-session is auto-respawned via
`Restart=on-failure`. The session is bootstrapped by
`~/.local/bin/sway-session` (run via `exec $bin/sway-session` in the
sway config), which propagates compositor env vars into dbus and
systemd before starting `sway-session.target`. The previous
`exec_always pkill -x kanshi; kanshi` autostart and the bash
`kanshi-restart` / `kanshi-boot-kick` helpers were retired — kanshi
1.7.0's subscribe-before-ready race is now closed at the systemd
dependency-graph level (kanshi.service `After=graphical-session.target`)
rather than papered over with bash retries.

**Audio stack (desktop role):** PipeWire-only — no PulseAudio
daemon. The desktop role installs `pipewire-pulse` (the libpulse
API shim — most apps speak Pulse, not native PipeWire),
`alsa-utils` (`alsamixer`, `aplay`, `arecord`, `speaker-test` —
required for any meaningful "no audio" debugging),
`libspa-0.2-bluetooth` (the BlueZ↔PipeWire bridge — without it
BT devices pair but produce no PipeWire sink), `rtkit` (lets
PipeWire request realtime scheduling priority — without it audio
crackles under CPU load), `wireplumber` (session manager), and
`fonts-noto-color-emoji` (emoji codepoints render as tofu boxes
without it). Two post-commands wire it together:
`systemctl --user enable --now pipewire-pulse.socket` (libpulse
clients connect via `$XDG_RUNTIME_DIR/pulse/native`; installing
the package is *not* enough, the socket needs explicit
enablement) and `sudo systemctl enable --now bluetooth.service`
(Debian's `bluez` postinst doesn't always enable it on minimal
installs). The `desktop_role_provisions_audio_stack` test in
`embed.rs` pins all four packages and both enable post-commands
together; if you split this up the test will catch it.

One non-obvious gotcha: **wireplumber needs a restart on existing
installs after `libspa-0.2-bluetooth` lands.** Wireplumber loads
its SPA monitor list at startup, so adding the BT bridge after
the session is already up leaves wireplumber blind to BT devices
until `systemctl --user restart wireplumber`. Fresh installs
don't hit this (wireplumber starts after the package is on disk)
which is why we don't add a forge post-cmd for it — but a one-
time `systemctl --user restart wireplumber` is needed when
upgrading an already-running session into BT support. Pairing
itself is per-device and manual; forge can't (shouldn't)
automate it.

**Screenshot key (desktop role, Print / Shift+Print):** one
script (`dotfiles/.local/bin/screenshot`) handles both modes,
sway just dispatches to it. `Print` calls `screenshot select`
(region selection via slurp), `Shift+Print` calls
`screenshot full` (all outputs). Both save to
`~/Pictures/screenshot_<UTC timestamp>.png` AND copy the same
bytes to the wl-clipboard, with a single notification covering
both. Esc during region selection is a silent cancel (no
"screenshot failed" notification — the user knows they
cancelled). To add a new mode (e.g. focused window): add a case
branch in the script and a new bindsym in
`dotfiles/.config/sway/config`. The
`screenshot_script_and_binds_ship_correctly` test asserts
the script ships, both binds resolve to the script, and the
legacy inline `$mod+(Shift+)Print` binds are NOT present —
those were removed when the script consolidated everything
into one bind per mode.

**Tweak the dictation stack (desktop role, Mod+s):** the model, the
binary, and the systemd unit are spread across three files — change
in lockstep or the `dictation_dotfiles_ship_correctly` and
`desktop_role_provisions_whisper_dictation_stack` tests will catch
the drift. To switch model size: change the URL on the
`whisper-small-en-q5_1-model` download in `config/roles.toml` and
the matching `--model` path in
`dotfiles/.config/systemd/user/whisper-server.service`. To change
the loopback port: update both the unit's `--port` and the
`WHISPER_PORT` constant in `dotfiles/.local/bin/dictate` (the test
asserts the two agree). For refreshing the vendored whisper-server
binary itself, see "Refreshing the vendored whisper-server binary"
below.

**Refreshing the vendored whisper-server binary:** whisper-server
and whisper-cli are checked into the dotfile tree at
`dotfiles/.local/bin/{whisper-server,whisper-cli}` as statically
linked prebuilt ELF binaries (~3 MiB each, stripped). They land on
hosts via the planner's ELF detection in `crates/common/src/planner.rs`,
which sets mode 0o755 on any embedded file starting with `\x7fELF`.
This replaces the previous host-side build chain — desktop role no
longer installs `build-essential` / `libopenblas-dev`, and the
`server` role installs `libopenblas0` + `libgomp1` (the runtime
libraries the binary still dynamically links against) so every
machine can run dictation without a build toolchain. To refresh:

```bash
forge dev up whisper-build       # spin up a clean dev container
# inside:
sudo apt install -y build-essential cmake libopenblas-dev
git clone --depth=1 https://github.com/ggml-org/whisper.cpp /tmp/w
cd /tmp/w
cmake -B build -DBUILD_SHARED_LIBS=OFF -DGGML_BLAS=1 -DGGML_NATIVE=ON \
      -DCMAKE_BUILD_TYPE=Release
cmake --build build -j --target whisper-server whisper-cli
strip build/bin/whisper-server build/bin/whisper-cli
exit
# back on host:
podman cp forge-dev-whisper-build:/tmp/w/build/bin/whisper-server \
          ~/linux-config/dotfiles/.local/bin/whisper-server
podman cp forge-dev-whisper-build:/tmp/w/build/bin/whisper-cli \
          ~/linux-config/dotfiles/.local/bin/whisper-cli
cargo install --path crates/forge --force
```

Three flags are load-bearing: `-DBUILD_SHARED_LIBS=OFF` (without it
whisper-server links to libwhisper.so + libggml*.so from the build
dir, which won't be on any other host); `-DGGML_BLAS=1` (routes
encoder GEMMs through OpenBLAS — measurable speedup on the encoder
GEMM kernels); `-DGGML_NATIVE=ON` (lights up AVX2/AVX-512 on Tiger
Lake — fine because every host running this binary is the user's
own laptop with the same arch family). If you start running this
on a heterogeneous fleet, drop NATIVE and pin a baseline ISA
explicitly.

## Dictation (Mod+s)

Local Whisper push-to-toggle dictation. Three pieces, all running as
the user, all lazy:

1. **`dictate` script** (`dotfiles/.local/bin/dictate`) is the
   keybind target. PID-file gated `pw-record` captures 16 kHz mono
   s16 WAV into `$XDG_RUNTIME_DIR/dictate/`. On the second press it
   SIGINTs the recorder (so pw-record finalizes the WAV header —
   SIGKILL would leave a truncated file unreadable by the server),
   POSTs the WAV to the local whisper-server, and types the
   transcription directly into the focused window via `printf '%s'
   "$text" | wtype -`. Subcommands: `cancel` (abort in-flight),
   `stop-server` (free the model RAM).
2. **`whisper-server.service`** is a `--user` systemd unit that
   binds the small.en-q5_1 model on `127.0.0.1:8127`. **It has no
   `[Install]` section by design** — the dictate script lazy-starts
   it on first dictation, polls the port for up to 30 s during cold
   model load, and the user stops it with `dictate stop-server`
   when the ~330 MiB of resident RAM is needed elsewhere. Adding an
   `[Install]` section would let `systemctl --user enable` succeed
   and silently waste that RAM on every login. The
   `dictation_dotfiles_ship_correctly` test enforces both that no
   `[Install]` section ships and that the unit's `--model` arg
   references the same GGML file the role downloads.
3. **whisper-server / whisper-cli** ship prebuilt as vendored
   binaries in the dotfile tree
   (`dotfiles/.local/bin/{whisper-server,whisper-cli}`). They are
   statically linked against libwhisper / libggml so the only
   runtime deps are libopenblas + libgomp + libgfortran (all
   pulled in by the `server` role's `libopenblas0` + `libgomp1`
   packages). On `forge setup`, the planner's ELF detection
   (`crates/common/src/planner.rs`) writes them with mode 0o755.
   No host build chain, no `~/.local/src/whisper.cpp/`, no symlinks
   — `~/.local/bin/whisper-server` is the actual binary on disk.
   See "Refreshing the vendored whisper-server binary" above for
   how to rebuild them in a dev container when upstream releases
   a new version.

Four gotchas worth knowing:

1. **Don't enable Vulkan on Iris Xe.** The whisper.cpp 1.8.3 release
   notes' "12× Vulkan" headline measures Arc / Meteor Lake, not
   Tiger Lake's Xe-LP iGPU. Real benchmarks on this exact GPU
   (i5-1135G7 → discussions/2375) show only ~40 % over CPU+OpenBLAS
   for the cost of dragging in a Vulkan SDK at build time. Skip it.
2. **Capture at 16 kHz, not 48.** whisper.cpp expects 16 kHz mono
   s16 PCM. Passing 48 kHz works (server resamples) but spends CPU
   for no benefit. The `pw-record --rate 16000 --channels 1
   --format s16` invocation in the dictate script is deliberate.
3. **`--threads 4`, not `--threads 8`.** This i7-1185G7 has 4
   physical cores and 8 hyperthreads. BLAS GEMM kernels are
   cache-bound, and a hyperthread pair shares L1/L2 — using all 8
   logical CPUs measurably slows the encoder vs. matching the
   physical core count. If you ever move this stack to a CPU with a
   different topology, retune the unit's `--threads` arg.
4. **Don't synthesize Ctrl+V to paste — type directly.** We tried
   `wl-copy + wtype -M ctrl v -m ctrl` and it broke in terminals:
   `foot` binds paste to Ctrl+Shift+V, `tmux` interprets Ctrl+V as
   a verbatim-insert prefix, and zsh's `quoted-insert` swallows it
   too. Synthesizing one chord can't satisfy every host app.
   `wtype -` typing the characters works uniformly and leaves the
   user's clipboard untouched, which is also what they want.
5. **Wait for samples before showing "Listening".** PipeWire takes
   ~100-300 ms after `pw-record` exec to actually open the capture
   stream and start writing samples. Without an explicit gate the
   "🎤 Listening" notification fires before audio is flowing, the
   user starts talking, and the first word or two get dropped.
   `start_recording` polls the WAV file size past the 44-byte
   header (50 ms ticks, 1.5 s cap) before notifying — that file
   only grows once pw-record has bound the stream, so it's a
   reliable "ready to capture" signal. The 1.5 s cap exists so a
   missing audio device or wedged PipeWire surfaces as a clear
   notification instead of a silent infinite wait.

To pick a different model size or quant, see "Tweak the dictation
stack" in Common tasks.

**Adjust the virt menu (server role + desktop role,
`$mod+c`):** unified fuzzel picker for podman containers, incus
LXCs, and libvirt-managed KVM VMs. Three pieces, all touched
together:

1. **`vm-menu` script** (`dotfiles/.local/bin/vm-menu`) — the
   keybind target. Snapshots every instance once at launch
   (no live refresh), renders fixed-width rows in fuzzel, and
   on selection pops a second fuzzel with the actions valid
   for the picked instance's current state (`start`, `stop`,
   `resume`, `shell`). The shell action auto-starts a stopped
   instance first, polls until running, then opens a new foot
   window dropped into `bash`/`sh`/serial console as
   appropriate.
2. **`server` role packages + post-cmds** — `incus`,
   `incus-client`, `qemu-system-x86`, `qemu-kvm`,
   `libvirt-daemon-system`, `libvirt-clients`, `bridge-utils`,
   `virtinst`, plus group adds (`incus-admin`, `incus`,
   `libvirt`, `kvm`), `systemctl enable --now libvirtd`,
   `virsh net-define /etc/libvirt/qemu/networks/default.xml`,
   `virsh net-autostart default && virsh net-start default`,
   and `incus admin init --auto --storage-backend=dir`. The
   `server_role_provisions_virt_stack` test pins all of these
   together; if you split a step out the test will catch it.
   The explicit `net-define` is defensive — Debian's
   `libvirt-daemon-system` postinst usually auto-imports the
   default network XML on first start, but on at least one host
   we caught it with the XML on disk yet absent from `virsh
   net-list --all`, which silently breaks the subsequent
   `net-autostart` / `net-start` post-cmds. Pinned by
   `server_role_defines_libvirt_default_network`.
3. **`desktop` role addition** — the virt-console GUI tier:
   `virt-manager` (the heavyweight management UI),
   `gir1.2-spiceclientgtk-3.0` (the SPICE GTK widget bindings —
   a Recommends of virt-manager that our `--no-install-recommends`
   apt strips, without which the SPICE viewer pane errors with
   "Namespace SpiceClientGtk not available"), and `virt-viewer`
   (ships the `remote-viewer` CLI that `incus console <name>
   --type=vga` shells out to — without it, the SPICE display
   silently fails to open). All three pinned together by
   `desktop_role_provisions_virt_console_tools`. Lives on
   desktop only because the headless CLIs cover what `vm-menu`
   needs; the GUI tier is the SPICE/VNC fallback for guests
   without a serial console.

Three gotchas worth knowing:

1. **Group memberships need a re-login to take effect.** The
   first `forge setup` after this stack lands runs `usermod
   -aG` for `incus`/`libvirt`/`kvm`/`incus-admin`, but the
   user's *current* shell session won't pick those up — group
   membership only attaches to processes started after the
   next login. Until then, `vm-menu` will pop the empty popup
   with the message `log out + back in to activate
   incus/libvirt groups` (the runtime health hint detects the
   permission-denied stderr and surfaces it). `newgrp libvirt`
   in the shell vm-menu was launched from is the cheapest
   workaround if a logout is impractical.
2. **`virsh console` requires a serial pty in the guest XML
   AND a getty running on it.** Modern distros installed via
   `virt-install` get this for free, but legacy/minimal images
   often don't, in which case the foot window opens, prints
   `Escape character is ^]`, and sits dead. The `do_shell`
   path in `vm-menu` notifies the user about this gotcha
   pre-emptively when the shell action targets a VM, so a
   silent-hang failure mode at least gets a hint of "try
   virt-manager instead". We deliberately do NOT fall back to
   ssh — picking a username and writing an ssh-config from
   the menu is too much state.
3. **The `incus admin init` post-command's `check` is
   load-bearing, not just a perf optimization.** `incus admin
   init` errors out on rerun against an already-initialized
   daemon (`existing instances or images. 'incus admin init'
   requires an empty database`), so without the
   `test -d /var/lib/incus/database` guard every forge rerun
   would fail at this step. The
   `server_role_provisions_virt_stack` test asserts the check
   is present.

To rebuild the keybind script after edits: `cargo install
--path crates/forge --force` (since the dotfiles tree is
embedded into the binary at compile time).

**Networking (one-manager rule, NetworkManager-only):**
NetworkManager owns every device on the box. Wifi has been
NM-managed since the start; ethernet was migrated off ifupdown +
dhcpcd because the split-manager arrangement made the
route-metric drop-in dead config (ethernet stayed `unmanaged`
per `nmcli device status` and dhcpcd's default metric=1006 won
the route race against NM's wifi at 600 — symptom: wired plugged
in, traffic still on wifi). Three migration post-cmds in the
desktop role implement the unification (search the role for
`05-ifupdown-managed.conf`):

1. `/etc/NetworkManager/conf.d/05-ifupdown-managed.conf` writes
   `[ifupdown] managed=true`, overriding Debian's shipped
   `[ifupdown] managed=false` in `/etc/NetworkManager/NetworkManager.conf`
   without editing the package-owned main file.
2. `/etc/network/interfaces` is rewritten down to loopback-only
   with a `# Managed by forge` marker the post-cmd's check
   greps for. Removes the brittle MAC-derived `enxXXXXXXXXXXXX`
   ethernet hardcode the Debian installer wrote — different USB
   ethernet dongle / dock = NM auto-detects, no edit.
3. `systemctl disable --now networking.service` so ifupdown
   stops competing for any device.

A separate post-cmd writes
`/etc/NetworkManager/conf.d/10-ethernet-priority.conf` with
`[connection-ethernet]` / `[connection-wifi]` `match-device=type:`
blocks that pin `ipv4.route-metric` (100 vs 600). NM's compiled-in
defaults already match these numbers; pinning them on disk guards
against a silent NM upstream change or a stray
`nmcli connection modify` flipping the preference. The match
syntax `match-device=type:ethernet` is documented in
`man NetworkManager.conf` under `[connection-*]`.

**`connection.autoconnect-priority` is intentionally NOT in the
drop-in.** Per `nm-settings(5)`, that property only chooses
between profiles competing for the *same device* (e.g. two saved
wifi networks on `wlan0`). It does not influence cross-device
preference — ethernet and wifi auto-connect independently when
their carriers come up. An earlier version of the drop-in pinned
`autoconnect-priority=100` / `-10` between the two device types
and that was dead config. Debian's NM also auto-generates "Wired
connection 1" with `autoconnect-priority=-999` (NM's default-
wired sentinel — set so any user-created wired profile at the
default 0 outranks it on the *same* device); the property only
matters per-device, so this is fine and safe to ignore.

**Exclusive failover via dispatcher script (real "ethernet when
plugged, wifi otherwise"):** `/etc/NetworkManager/dispatcher.d/10-wifi-when-no-ethernet`
disconnects every wifi device when an `en*`/`eth*` device
transitions to `up`, and reconnects every wifi device when the
last ethernet device transitions to `down`. The route-metric pin
above only fixes *kernel routing* — it does NOT fix
`getaddrinfo()` / `connect()` source-address selection per RFC
6724, which on a same-subnet wifi+ethernet host (laptop docked
into the same LAN as the wifi AP) picks the lower-ifindex
interface (usually wifi) for outbound binds even when ethernet
has the lower route metric. Symptom: VPN client / `curl
ifconfig.me` reports the wifi public IP despite the wire being
plugged in. The dispatcher script removes the wifi IP entirely
when ethernet is up, so no app can pick it. Uses `nmcli device
disconnect` (not `radio off`) so the wifi card still scans and
the user can manually pick a network with `nmcli device connect
<wifi-dev>` to override the dispatcher. Loop-prevention is
trivial: the script's first `case` filters on `en*`/`eth*`, so
the wifi-up event NM fires when we re-enable wifi doesn't
recurse. Pinned by `desktop_role_installs_wifi_failover_dispatcher`.

**Migration gotcha** (only matters when forge first runs on a
box that previously had `[ifupdown] managed=false`): NM caches
the device's unmanaged decision and `nmcli general reload conf`
re-reads the config file *without* re-evaluating already-
unmanaged devices. Plus `disable --now networking.service`
doesn't reliably kill the orphaned dhcpcd children (Debian's
unit's ExecStop is sometimes a no-op). The clean fix is to
**reboot**: `networking.service` is disabled so ifupdown won't
restart, NM starts fresh with `managed=true` in effect from
boot, dhcpcd never gets respawned. Fresh boxes pick up the
unified setup from first boot — no reboot needed because
nothing was holding the device wrongly to begin with. Pinned by
`desktop_role_unifies_networking_under_nm` and
`desktop_role_pins_networkmanager_ethernet_priority`.

**Auto-updates (apt + flatpak, no auto-reboot):** the box keeps
itself patched without manual `apt upgrade` cycles. Two halves,
both opt-in via forge:

- **apt** (server role, so every host inherits): installs
  `unattended-upgrades` and writes two drop-ins.
  - `/etc/apt/apt.conf.d/20auto-upgrades` enables the four
    periodic flags (`Update-Package-Lists`,
    `Download-Upgradeable-Packages`, `Unattended-Upgrade`,
    `AutocleanInterval`) — without these,
    `apt-daily-upgrade.timer` fires daily and silently does
    nothing because the package's debconf prompt on a
    non-interactive install never sets them.
  - `/etc/apt/apt.conf.d/52unattended-upgrades-local.conf`
    extends the package's shipped `50unattended-upgrades`
    (loaded after via apt's alphabetical merge):
    `Origins-Pattern::` appends the Debian stable + updates
    suites so point-release fixes land too (not just
    security); `Automatic-Reboot "false"` is pinned (default
    is already false; pin guards against future flip);
    `MinimalSteps "true"` so an interrupted run resumes
    cleanly instead of leaving a half-configured package.
- **flatpak** (desktop role only — only desktop installs
  flatpaks): writes
  `/etc/systemd/system/flatpak-update.{service,timer}` running
  `flatpak update -y --noninteractive` daily as root. System
  flatpak installs live under `/var/lib/flatpak` (root-owned),
  so a per-user timer can't update them. The timer is
  `OnCalendar=daily` + `RandomizedDelaySec=30m` +
  `Persistent=true`, ordered after `network-online.target`.

**The flatpak-update.service is hardened** with eleven systemd
sandbox directives — `NoNewPrivileges=yes`, `PrivateTmp=yes`,
`ProtectSystem=strict` + `ReadWritePaths=/var/lib/flatpak`,
`ProtectHome=read-only`, `ProtectKernel{Tunables,Modules}=yes`,
`ProtectControlGroups=yes`, `ProtectClock=yes`,
`LockPersonality=yes`, `RestrictRealtime=yes`. The unit runs as
root because system flatpaks require it; the directives
contain the blast radius if flatpak (or any of its deps) ever
has a privilege-escalation bug. Pinned by
`desktop_role_hardens_flatpak_update_service`. We deliberately
do NOT add `RestrictNamespaces=` because flatpak's update path
uses bwrap for triggers, and over-constraining namespaces
breaks updates with errors that are hard to diagnose. No
auto-reboot anywhere — kernel reboots stay user-initiated.
Verify timers fire with `systemctl list-timers
apt-daily-upgrade.timer flatpak-update.timer`.

**Host firewall (server role, deliberately none):**
`/etc/nftables.conf` is a forge-managed file with the marker
comment `# Managed by forge: intentionally empty` and no
ruleset. The path is owned (so an ad-hoc edit can't squat
it), but the contents are pure documentation — no tables, no
chains, no policy. `nftables.service` is left as Debian
shipped it (loaded but inert against an empty config); the
service isn't disabled because netavark / libvirt / incus all
program nftables for their own bridges through the same
kernel subsystem, and pulling the package or masking the
service would surprise those daemons.

An earlier version of this role created an `inet forge_filter`
table with `policy drop` on input plus a small allowlist
(loopback, established/related conntrack, ICMP/ICMPv6, ssh).
That looked correct on paper but caused real problems:
**rootless podman + pasta UDP DNS replies were not reliably
matching `ct state established,related accept`** in the
input chain on this kernel/pasta combo. Symptom: `forge dev
build` failed at the very first `apt-get update` with
`Temporary failure resolving 'deb.debian.org'` even though
the host could curl the same URL fine. Reproducible with
`podman run --dns=10.10.20.1 debian:trixie-slim getent hosts
deb.debian.org` (fails) vs `--network=host` (succeeds on the
same machine, same DNS, same network).

The choice to roll the firewall back rather than work around
the pasta interaction:

- This is a single-user developer laptop behind a NAT
  router. Inbound traffic from the public internet is already
  filtered at the router; default-drop on input was layering
  on top of that for marginal benefit.
- Debian services bind to `127.0.0.1`/`::1` by default; the
  ones that *do* listen LAN-wide (sshd, libvirt's bridge for
  VM consoles) are the ones we *want* reachable.
- The LAN itself is trusted (home network).
- Distro defaults — Debian, Ubuntu, Fedora workstation — ship
  with no host-level filtering on a fresh install. Our
  forge_filter was extra complexity with a real cost (broke
  rootless podman) and no real upside.

**The negative invariant against `flush ruleset` is still
load-bearing.** Debian's stock `/etc/nftables.conf` opens
with `flush ruleset`, and even our otherwise-empty file
would wipe netavark / libvirt / incus tables on every
`systemctl reload nftables` if we added that line.
`server_role_provisions_host_firewall` walks every
post-command in both roles and fails if `flush ruleset`
appears anywhere. If you're rewording a comment in the
firewall body, keep the literal phrase out of it or the test
fails. The original `forge_filter` ruleset is preserved in
git history if a future host (real server, public-facing)
ever needs filtering — restore it via a per-host drop-in,
NOT by re-introducing it to the base server role, and
investigate the pasta interaction first.

**Power profiles (desktop role, laptop):**
`power-profiles-daemon` exposes `power-saver` / `balanced` /
`performance` over D-Bus, switchable at runtime via
`powerprofilesctl set <profile>`. Tweaks Intel P-state EPP,
platform_profile (when the firmware exposes one — Tiger Lake
i7-1185G7 does), and CPU governor in lockstep. Pure system
service, no DE-specific bits, works the same under sway. Default
profile on install is `balanced`.

We deliberately don't pin a default profile or auto-switch
in the role — auto-switching on AC plug/unplug is a polish
task you wire up via a UPower dispatcher script if you want
it (PPD doesn't auto-switch by design). Lives in desktop
only, not server, because every knob it exposes is laptop-
shaped — a server-on-AC just idles in `balanced` for ~5 MiB
of nothing. Pinned by
`desktop_role_provisions_power_profiles_daemon`. If
`powerprofilesctl get` returns "No D-Bus name
net.hadess.PowerProfiles" the daemon's not running —
`systemctl status power-profiles-daemon` will tell you why.

## Things that look load-bearing but aren't

- **`legacy/`** is a reference snapshot of the previous Ansible/Stow/Python
  implementation. Don't extend it. If you find functionality that wasn't
  ported (look in `legacy/bin/`, `legacy/ansible/`, `legacy/docs/`),
  port it into the Rust workspace properly — embed scripts in
  `dotfiles/.local/bin/`, encode install steps in `config/roles.toml`.
- **`README.md`** describes what the project is for users; **CLAUDE.md** (this
  file) is the authoritative architectural doc for AI/agent work.

## Design principles

- One binary, no runtime config lookup, no implicit network.
- Idempotent at every layer; a rerun is a cheap no-op.
- Tests catch broken embedded assets at `cargo test`, not at install.
- Sway/Wayland only. Keyboard-driven. No X11.
- Optimize for "fresh box → working desktop in one command".
