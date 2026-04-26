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

**Roles:** `server`, `dev`, `desktop`, `remote-desktop`, `net-box`, `attack-box`.

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
│   ├── dev-images.toml     # podman dev container runtimes
│   └── vm-profiles.toml    # incus VM profiles
├── themes/
│   ├── config.toml         # template targets + scheme metadata
│   ├── schemes/*.yaml      # base16 color schemes
│   └── templates/*.j2      # tera templates rendered per-app
├── containers/             # Containerfiles for dev images
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
   - `Download` per binary (nvim, starship, yazi, fonts...)
   - `WriteFile` per dotfile that matches a glob — files starting with `#!`
     get mode `0o755` so embedded scripts execute
   - `PostCommand` per `post` entry
3. **Execute** (`forge_common::executor::execute`): every action is idempotent.
   `is_installed`, content-equality checks in `write_atomic`, `dest.exists()`
   for downloads, and the optional `check` field on post-commands all let
   reruns be cheap no-ops.
4. **State** is written to `~/.local/state/forge/state.json` so `forge status`
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
either rerun `forge setup desktop` or — for a hot reload without going
through apt — copy the file to `~/.config/kanshi/config` and
`pkill -HUP kanshi`. Sway's `exec_always pkill -x kanshi; kanshi`
respawns the daemon on every `swaymsg reload`, so the next reload also
re-applies cleanly.

**Tweak the dictation stack (desktop role, Mod+s):** the model, the
build flags, and the systemd unit are spread across three files —
change in lockstep or the `dictation_dotfiles_ship_correctly` and
`desktop_role_provisions_whisper_dictation_stack` tests will catch
the drift. To switch model size: change the URL on the
`whisper-medium-en-model` download in `config/roles.toml` and the
matching `--model` path in
`dotfiles/.config/systemd/user/whisper-server.service`. To change the
loopback port: update both the unit's `--port` and the
`WHISPER_PORT` constant in `dotfiles/.local/bin/dictate` (the test
asserts the two agree). To rebuild whisper.cpp from a newer upstream:
`rm -rf ~/.local/src/whisper.cpp` and rerun `forge setup desktop` —
the clone/build post-commands are guarded by checks that probe for
the source dir and the built binary respectively, so a missing tree
re-triggers them.

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
3. **whisper.cpp** is built from source during `forge setup
   desktop` (no official Linux binaries upstream). The post-command
   uses `cmake -B build -DGGML_BLAS=1 -DGGML_NATIVE=ON --target
   whisper-server whisper-cli` — both flags matter: BLAS routes
   encoder GEMMs through OpenBLAS (libopenblas-dev), NATIVE lights
   up AVX2/AVX-512 on Tiger Lake. Source lives in
   `~/.local/src/whisper.cpp/`; `~/.local/bin/whisper-server` and
   `whisper-cli` are symlinks into the build dir so the systemd
   unit can use a stable short path.

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
