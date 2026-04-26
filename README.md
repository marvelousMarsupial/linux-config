# linux-config

Single-binary Linux workstation provisioner. One Rust binary embeds the dotfile
tree, role definitions, and theme system at compile time — copy `forge` onto a
fresh Debian box and `forge setup desktop` brings it from stock to fully
configured in one command.

## Quick start

```bash
git clone <this-repo> ~/linux-config
cd ~/linux-config
cargo build --release -p forge
~/linux-config/target/release/forge setup desktop
```

The binary is self-contained — no repo clone required at runtime, no separate
config files to manage. Drop `target/release/forge` onto any Debian box and it
has everything it needs.

## Roles

```
server                  base CLI: zsh, tmux, neovim, yazi, starship, theme
├── dev                 + build tools, nodejs, lazygit
├── desktop             + sway, foot, fuzzel, bin scripts, zen, obsidian
│   ├── remote-desktop  + tigervnc
│   └── attack-box      + wireshark, nmap, sqlmap, hydra, ...
└── net-box             + tshark, suricata, nmap, dns tools
```

Each role inherits its parent's packages, dotfiles, and post-install commands.
Definitions live in [`config/roles.toml`](config/roles.toml).

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

forge dev up <name>          # podman dev container
forge vm create <profile>    # incus VM
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
│   ├── common/         # forge_common: planner, executor, theme, state
│   └── forge/          # binary; embeds dotfiles/ + config/ + themes/
├── dotfiles/           # mirrors $HOME (.zshrc, .config/**, .local/bin/**)
├── config/roles.toml   # role definitions
├── themes/             # base16 schemes + tera templates + targets config
├── containers/         # Containerfiles for dev images
└── legacy/             # previous Ansible/Stow implementation (reference only)
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
