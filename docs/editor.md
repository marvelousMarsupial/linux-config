# Neovim

[Documentation index](README.md)

## Setup

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
download defaults to v0.12.2 and verifies a pinned SHA-256 checksum before extraction.
Overriding `NVIM_VERSION` also requires a verified `NVIM_SHA256`.
[Official release digests](https://github.com/neovim/neovim/releases/expanded_assets/v0.12.2).

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

## Markdown view

Space+mr toggles the existing in-buffer Markdown view. Rendering depends on the
Markdown Treesitter parser; optional parser setup remains part of `--dev-tools`.
Plain text and Markdown paragraphs remain unchanged on save.

Browser preview (`MarkdownPreview` and Space+mp) is disabled. Its former build
hook downloaded and extracted an external helper without verifying a checksum,
and could execute an existing helper while checking its version. Disabling the
plugin prevents the normal command/filetype paths from loading that helper;
the build hook has also been removed. The installed plugin and binary were not
deleted or claimed verified. Restart Neovim to apply the installed config; already
running editor/preview processes are unaffected. Revisit browser preview only
with a verified installation path if the feature is wanted.
