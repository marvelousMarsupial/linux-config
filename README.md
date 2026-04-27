# linux-config

Single-binary Linux workstation provisioner. One Rust binary embeds the
dotfile tree, role definitions, theme system, the dev container's
Containerfile, and the vendored prebuilt whisper-server binary at
compile time — copy `forge` onto a fresh Debian box and
`forge setup desktop` brings it from stock to fully configured in one
command. No repo clone required at runtime, no rust toolchain required
on the host, no implicit network fetches outside a small set of
explicit upstream binary downloads.

## Quick start

**Build once** (on any machine with a Rust toolchain — typically a dev
container, see "Rebuilding `forge`" below):

```bash
git clone <this-repo> linux-config
cd linux-config
cargo build --release -p forge
```

**Run anywhere** — copy `target/release/forge` to a fresh Debian 13
host and:

```bash
./forge setup desktop      # or: forge setup server (for headless boxes)
```

The host doesn't need git, doesn't need cargo, doesn't need this repo.
Just the single binary and passwordless sudo.

## Rebuilding `forge`

The host stays Rust-free by design. To rebuild the binary, do it inside
a dev container (which has rustup baked into the image) and copy the
result out:

```bash
forge dev up forge-build
# inside the container:
git clone <this-repo> /tmp/lc && cd /tmp/lc
cargo build --release -p forge
# back on host:
podman cp forge-dev-forge-build:/tmp/lc/target/release/forge ~/.local/bin/forge
```

## Roles

```
server      base CLI: zsh, tmux, neovim, yazi, starship, theme,
            rootless podman + companions, foot terminfo (ncurses-term),
            whisper.cpp runtime libs (libopenblas0, libgomp1)
└── desktop + sway, foot, fuzzel, bin scripts, zen, obsidian,
              audio + dictation stack, lazygit, cmake
```

Two roles. `desktop` extends `server`, picking up everything `server`
ships and layering the GUI + dictation pieces on top. Definitions live
in [`config/roles.toml`](config/roles.toml).

## Commands

```bash
forge setup <role>           # install
forge setup <role> --dry-run # show the plan, change nothing
forge setup <role> --diff    # plan + per-dotfile unified diff
forge switch <role>          # teardown current → setup new
forge teardown               # remove what the current role installed
forge status                 # current role, theme, applied timestamp

forge theme list             # available base16 schemes
forge theme apply <name>     # render theme across every configured target
forge theme current          # show active scheme

forge dev build              # build the dev container image (one-time)
forge dev up <name>          # create + attach to a per-project container;
                             #   creates fresh, starts if stopped, attaches
forge dev attach <name>      # re-attach to a running container
forge dev exec <name> -- cmd # run one-off command in container
forge dev down <name>        # stop (state preserved in <name>-home volume)
forge dev rm <name>          # remove the container (volume survives)
forge dev ls                 # list dev containers
```

Dev pods are one container per project. Same shell, editor, and AI
tooling baked into the image; per-project toolchains (rustup, fnm,
uv, pyenv) installed inside and persisted in the named volume so
they survive `forge dev rm`. The image is built from
`containers/base.Containerfile` (embedded into the binary), so
`forge dev build` works without the repo on disk.

To install foot's terminfo on a remote you don't manage, the `fix`
helper (defined in `dotfiles/.zsh_aliases`) pushes it via SSH:

```bash
fix me@vps.example.com       # one-off; persists in the remote's ~/.terminfo
```

## Desktop keybinds (Sway, `Super` = Mod)

| Key | Action |
|---|---|
| `Super+Return` | foot terminal |
| `Super+Space` | fuzzel app launcher |
| `Super+b` | Zen browser |
| `Super+n` | new vim note (foot + nvim) |
| `Super+m` | search notes (fuzzel picker over ~/vault) |
| `Super+Shift+n` | Obsidian |
| `Super+e` | yazi file manager |
| `Super+v` | nvim |
| `Super+p` | password picker |
| `Super+s` / `Super+Ctrl+s` | dictate (toggle / cancel) — local Whisper |
| `Super+x` | power menu |
| `Super+w` | wifi menu |
| `Super+i` / `Super+Shift+i` | status panel / btop |
| `Print` / `Shift+Print` | screenshot region / fullscreen — saves to `~/Pictures` and copies to clipboard |
| `Super+Shift+/` | keybindings cheatsheet |
| `Super+Escape` | lock |
| `Super+Shift+c` | reload sway |

`Super+h/j/k/l` focus, `Super+Shift+h/j/k/l` move, `Super+1..9` workspaces.
Run `keybinds` for the full list.

## Theme system

Base16 colors applied across foot, sway, fuzzel, mako, swaylock, gtk3/4, tmux,
neovim, yazi, obsidian, zen browser. Schemes: `gruvbox-dark-hard`,
`catppuccin-mocha`, `kanagawa`, `tokyonight-moon`, `onehalfdark-black`,
`summer-dusk`.

```bash
forge theme apply catppuccin-mocha
```

## Repo layout

```
linux-config/
├── crates/
│   ├── common/             # forge_common: planner, executor, theme, state
│   └── forge/              # binary; embeds dotfiles/ + config/ + themes/
│                           #   + Containerfile + vendored whisper bins
├── dotfiles/               # mirrors $HOME (.zshrc, .config/**, .local/bin/**)
│                           # .local/bin includes vendored whisper-server +
│                           # whisper-cli (statically linked, ~3 MB each)
├── config/
│   ├── roles.toml          # role definitions
│   └── dev-container.toml  # single dev image config (image_tag, ports, …)
├── themes/                 # base16 schemes + tera templates + targets config
├── containers/
│   └── base.Containerfile  # the one dev image; embedded at compile time
│                           # so `forge dev build` works from any cwd
└── legacy/                 # previous Ansible/Stow implementation (reference only)
```

## Contributing

This project is the author's personal workstation config — pull requests aren't
expected, but if you fork it, a few load-bearing rules:

- **Rustdoc on every `pub` item.** Modules get a `//!` header explaining their
  purpose; functions/types get `///` doc comments. `cargo doc --no-deps
  --workspace` should build clean and read like a self-contained reference.
- **Tests for every behavior.** New code lands with happy-path, error-path, and
  idempotency tests. Embedded-asset invariants (roles.toml parses, every theme
  template exists) are covered in `embed.rs::tests`. `cargo test --workspace`
  must stay green.
- **No new dependencies casually.** The workspace already pulls in what's
  reasonable for a binary this size.

See [`CLAUDE.md`](CLAUDE.md) for the architectural deep-dive.

## Requirements

- Debian 13+ (or compatible apt-based distro)
- Passwordless sudo for the user running `forge setup` (apt and a few symlink
  steps need root)

## License

MIT
