# linux-config

Personal Debian desktop configs and small scripts. Sway, Foot, Fuzzel,
Neovim, Yazi, zsh, and a shared Base16 theme. Virtual machines are managed
directly with libvirt; Super+c opens virt-manager.

## Install

Start with Debian 13 and your normal user account. Review the package list:

```sh
sudo apt-get update
sed '/^#/d; /^$/d' packages.txt | xargs sudo apt-get install --
./scripts/install --dry-run
./scripts/install
theme apply tokyonight-moon
```

The installer copies files into your home, backs up replaced files under
`~/.local/state/linux-config/backups/`, and skips identical files. It does
not install packages, change services, remove software, or alter networking.
Run it again after editing the repo. Existing monitor profiles, Neovim plugin
locks, and generated color files are preserved. `--home /path` installs into
an alternate home for testing. Copies are independent of the checkout.

For a fresh desktop, enable the existing user monitor service explicitly:

```sh
systemctl --user daemon-reload
systemctl --user enable kanshi.service
```

Open a new terminal after installation and press Super+Shift+c to reload Sway.
Choose zsh as your login shell with `chsh -s /bin/zsh` if desired. Sway starts
on tty1; run `sway` manually for an initial session.

## Neovim and development tools

Run the explicit editor bootstrap after cloning:

```sh
./scripts/setup-editor
```

The default installs a lightweight editor, IPython, jq, bat, and theme
requirements. It installs Neovim 0.12+ if needed, copies configs with backups,
and installs editor plugins. It does **not** install compilers, Node/npm,
language servers, formatters, or Treesitter parsers.

Development support is kept in the config for machines where you want it:

```sh
./scripts/setup-editor --dev-tools
```

That explicitly adds C build tools, Node/npm, language servers, formatters,
and parsers. The desktop package list excludes these development tools.
IPython is available as `ipython3`. Both `cat` and `bat` use Debian's batcat
with a complete rounded frame in interactive terminals. Pipes stay unchanged;
`batcat` remains available directly.
Existing tools are never automatically uninstalled. `--skip-apt` skips the
privileged package step. Downloads need access to GitHub and npm; the Neovim
download defaults to v0.12.2 and can be overridden with `NVIM_VERSION`.

The configured languages are C/C++, TypeScript/JavaScript/TSX, HTML/CSS,
JSON/JSONC, YAML, and Lua (including Neovim config). clangd uses your project's
`compile_commands.json`; with CMake, configure with
`-DCMAKE_EXPORT_COMPILE_COMMANDS=ON`. For other build systems, provide a compile
database or a `.clangd` file with your include paths and compiler flags.
TypeScript projects should include their normal project dependencies and
`tsconfig.json`; the editor's server doesn't replace those.

`:EditorTools` shows installed and missing executables; `:Mason` manages tools,
`:ConformInfo` explains formatting, and `:checkhealth vim.lsp` checks LSP health.
With optional development tools installed, use `gd` for definitions, `K` for docs, Space+rn to rename, Space+ca for code
actions, and Space+cf to format. Code formats on save when its formatter or language server is installed. Plain text and Markdown
always soft-wrap and keep paragraphs unchanged on save. `j`/`k` follow screen
lines unless given a count. Enter no longer accepts an unselected completion.

Space+vc opens Neovim's config; Space+vh opens LSP health; Space+vt lists tools.
Edit source configs in the repo and rerun `scripts/install` to preserve changes
for the next machine. Space+vc opens the live copy for quick experiments; copy
those edits back into the repo before reinstalling. Restart Neovim after changes.

Other personal applications remain separate: Yazi/ya, Starship, Zen, and the
CaskaydiaCove Nerd Font. Put tools in `~/.local/bin` or use your preferred package
source. Zen expects `~/.local/zen/zen`. Existing installations are left alone.
Debian calls bat/fd `batcat` and `fdfind`; bat/cat aliases handle batcat
automatically. Provide a local `fd` symlink if desired. Optional fnm integration uses `~/.local/share/fnm`.

The launcher includes entries for separately installed Flatpak applications.
Obsidian is not part of this setup; existing applications and notes stay intact.

## Themes

```sh
theme list
theme preview catppuccin-mocha
theme apply catppuccin-mocha
theme current
```

Six schemes remain available, including TokyoNight, Catppuccin, Kanagawa,
Gruvbox, One Half Dark, and Summer Dusk. Python 3.11+, `python3-jinja2`, and
`python3-yaml` are required. Schemes, templates, and target definitions live
in `dotfiles/.config/theme/` — one source of truth, no build step.

Theme application renders every template before writing and runs configured
application reload hooks. Existing Foot processes are never signaled; open a new terminal
for terminal colors. Reload Sway and restart open editors to finish applying colors.
An absent application may report that its reload is unavailable.

## Virtual machines

Use virt-manager, virsh, virt-install, and virt-viewer directly. The package
list includes the libvirt/QEMU tools, UEFI firmware, and SPICE support.
For a local hypervisor, configure the service and your access explicitly:

```sh
sudo usermod -aG libvirt,kvm "$USER"
sudo systemctl enable --now libvirtd
virsh --connect qemu:///system list --all
```

Log out and back in after changing groups. Configure networks and storage
through virt-manager as needed. Remote libvirt connections also work through
virt-manager. This repo does not create, start, or delete guests.

## Local and system configuration

`~/.config/sway/local.conf` is optional and never installed by the repo.
Use it for machine-specific bindings or output rules. Edit your kanshi
profiles locally; `examples/kanshi-laptop.conf` preserves the previous
physical laptop/monitor layout as a reference. The default kanshi config
contains no hardware assumptions.

`system/` mirrors selected `/etc` files for unattended apt updates and lid
behavior. Review and copy individual files with sudo only when wanted.
They are not automatically applied. Networking and remote access remain
under your control.

## Layout and checks

- `dotfiles/`: user configs, theme assets, and desktop scripts.
- `scripts/install`: backup and copy configs; supports dry runs.
- `scripts/check`: script syntax, TOML parsing, and all theme template renders.
- `scripts/setup-editor`: explicit editor and language-tool installation.
- `packages.txt`: explicit Debian package list.
- `system/`: optional system configuration files.
- `examples/`: hardware-specific reference configs.

Run `./scripts/check` before applying changes and `./scripts/check --full`
for the offline integration suite. Forge and the previous
provisioning/container implementations are retired; Git history preserves them.

Yazi requires the `file` package for MIME detection and correct preview/open behavior.
Optional preview tools: `poppler-utils` for PDFs, `ffmpeg` for videos, and `7zip` for archives.
See [Yazi dependencies](https://yazi-rs.github.io/docs/installation/).
