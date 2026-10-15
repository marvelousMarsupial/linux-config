# Neovim

[Documentation index](README.md)

## Setup

Run the explicit editor bootstrap after cloning:

```sh
./scripts/setup-editor
```

The default installs a lightweight editor, IPython, jq, bat and theme
requirements. It installs Neovim 0.12+ if needed, copies configs with backups
and installs editor plugins. It does **not** install compilers, Node/npm,
language servers, formatters or Treesitter parsers.

```sh
./scripts/setup-editor --dev-tools
```

That adds C build tools, Node/npm, language servers, formatters and parsers.
The desktop package list excludes these. `--skip-apt` skips the privileged
package step; full setup uses it. Existing tools are never uninstalled.

Downloads need access to GitHub and npm. The Neovim download defaults to
v0.12.2 and verifies a pinned SHA-256 checksum before extraction; overriding
`NVIM_VERSION` also requires a verified `NVIM_SHA256`.
[Official release digests](https://github.com/neovim/neovim/releases/expanded_assets/v0.12.2).
The lazy.nvim bootstrap checks out the exact commit recorded in the lock file.

IPython is available as `ipython3`. Both `cat` and `bat` use Debian's batcat
with a rounded frame in interactive terminals; pipes stay unchanged and
`batcat` remains available directly.

## Languages and tools

Configured languages: C/C++, TypeScript/JavaScript/TSX, HTML/CSS, JSON/JSONC,
YAML and Lua, including the Neovim config. clangd uses your project's
`compile_commands.json`; with CMake, configure with
`-DCMAKE_EXPORT_COMPILE_COMMANDS=ON`. For other build systems, provide a compile
database or a `.clangd` file. TypeScript projects need their normal
dependencies and `tsconfig.json`.

`:EditorTools` shows installed and missing executables, `:Mason` manages tools,
`:ConformInfo` explains formatting, and `:checkhealth vim.lsp` checks LSP health.
With development tools installed, `gd` goes to a definition, `K` shows docs,
Space+rn renames, Space+ca lists code actions and Space+cf formats. Code
formats on save when its formatter or language server is installed. Plain text
and Markdown always soft-wrap and keep paragraphs unchanged on save. `j` and `k`
follow screen lines unless given a count. Enter does not accept an unselected
completion.

Space+vc opens the live Neovim config, Space+vh opens LSP health and Space+vt
lists tools. Edit source configs in the repo and rerun `scripts/install` to
keep changes for the next machine; copy quick live experiments back into the
repo before reinstalling. Restart Neovim after changes.

Modelines and project-local editor config are disabled to keep untrusted files
from changing editor options.

## Operator symbols

Foot does not render programming ligatures, so Neovim conceals a few operators
as single Unicode symbols. Tree-sitter handles JavaScript, TypeScript, JSX, TSX
and C; Python uses Vim syntax conceal because the base setup does not install
its parser.

| Languages | Displayed as |
| --- | --- |
| JavaScript, TypeScript, JSX, TSX | `=>` as `⇒`, `===` as `≡`, `!==` as `≢`, `!=` as `≠`, `<=` as `≤`, `>=` as `≥` |
| C | `->` as `→`, `!=` as `≠`, `<=` as `≤`, `>=` as `≥` |
| Python | `->` as `→`, `!=` as `≠`, `<=` as `≤`, `>=` as `≥` |

The symbols are bold. Strings and comments stay literal, the cursor line shows
the real characters in every mode, and buffer text and saved files are
unchanged. Toggle with **Space u l** or `:LigaturesToggle`; the toggle lasts
for the session.

## Markdown view

Space+mr toggles the in-buffer Markdown view. Rendering depends on the Markdown
Treesitter parser, which is part of `--dev-tools`.

Browser preview (`MarkdownPreview` and Space+mp) is disabled. Its build hook
downloaded an external helper without verifying a checksum and could execute an
existing helper while checking its version. The plugin is disabled at its load
paths and the hook is removed; an already installed helper is left in place
and is not treated as verified. Revisit browser preview only with a verified
installation path.
