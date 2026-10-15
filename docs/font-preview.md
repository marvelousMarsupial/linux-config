# Font preview in Foot

Run from the repository:

```sh
./scripts/setup-font-preview
~/.local/bin/font-preview
```

The setup downloads FiraCode Nerd Font from the official Nerd Fonts release,
verifies its published SHA-256 with the existing setup helpers, installs it
under `~/.local/share/fonts/FiraCode`, and refreshes the font cache. No sudo is
needed. Repeat runs reuse an installed matching font and unchanged preview files.

The preview opens the existing Neovim configuration and a read-only sample in a
separate Foot window, with a font override for that window only. Compare text,
accents, icons and character shapes with a normal terminal. Use `:q` to close it.
FiraCode Nerd Font Mono is now the Foot default at size 11. New terminal
windows use it for the shell, Neovim and Yazi. Other desktop app fonts are
unchanged. Kitty is not involved.

Foot does not render programming ligatures. The `⇒`, `≡` and `≠` sample characters
are literal Unicode examples; operators in the sample code are unchanged.
Neovim uses Tree-sitter conceal for JavaScript/TypeScript (including JSX/TSX)
and C; Python uses Vim syntax conceal because its Tree-sitter parser is not
installed by the base setup. The replacements are:

| Languages | Operators displayed differently |
| --- | --- |
| JavaScript, TypeScript, JSX, TSX | `=>` → `⇒`, `===` → `≡`, `!==` → `≢`, `!=` → `≠`, `<=` → `≤`, `>=` → `≥` |
| C | `->` → `→`, `!=` → `≠`, `<=` → `≤`, `>=` → `≥` |
| Python | `->` → `→`, `!=` → `≠`, `<=` → `≤`, `>=` → `≥` |

The substituted symbols are bold for more weight. A terminal still uses one
font size per window, so individual operator glyphs cannot be enlarged. Strings
and comments stay literal. The entire cursor line reveals the original
characters in all modes. Buffer text and saved files are unchanged.

Toggle the symbols with **Space u l** or `:LigaturesToggle`. Restart Neovim
to load this configuration. The toggle lasts for that Neovim session. Other
filetypes retain their original conceal options.

The font is installed on the current machine and the user accepted the preview.
The setup command remains available for a fresh install. No live Neovim rendering
check was possible from the sandbox; query and toggle behavior passed headless tests.
