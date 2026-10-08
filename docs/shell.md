# Shell

[Documentation index](README.md)

The setup uses Zsh without a shell framework. Full setup installs its Debian
dependencies and the pinned fzf-tab plugin. Open a new terminal after updating
`.zshrc`.

## History and suggestions

| Action | Behavior |
| --- | --- |
| Type `git st`, then Up | Browse older commands starting with `git st`, such as `git status` or `git stash`; skip `git log`. |
| Down during prefix search | Browse newer matches; past the newest match, restore the original typed prefix. |
| Up/Down on an empty prompt | Browse all history. |
| Ctrl+R | Open fzf history search. Choose an entry to insert it for editing; Escape cancels. |
| Right Arrow at the end of the typed line | Accept the grey suggestion. |

Prefix search uses the text from the start of the line through the cursor.
The cursor stays at the prefix so repeated arrow presses keep the same search.
Normal and application arrow sequences work in Emacs and vi insert mode.

Autosuggestions use history first and fall back to Zsh's completion system when
history has no match. Completion can suggest paths or arguments you have not
typed before. Atuin and zsh-autocomplete are not part of this setup.

## Tab completion and previews

fzf-tab presents Zsh completion candidates in a searchable menu. Try `cd ` then
Tab, or `git ` then Tab. A unique completion may be inserted directly; press Tab
again if the first press only extends the common prefix. Enter selects a result
without executing the command; Escape cancels. Directory candidates show a
listing, and regular files show the first 200 lines through `batcat`.

For an existing installation, run from the checkout:

```sh
./scripts/setup-shell --dry-run
./scripts/setup-shell
./scripts/install
```

`scripts/install` copies the project's user configs with backups; preview that
step with `./scripts/install --dry-run`. `scripts/setup-shell` only installs
fzf-tab and requires GitHub access. It checks v1.2.0 against commit
`01dad759c4466600b639b442ca24aebd5178e799`, installs into
`~/.local/share/zsh/fzf-tab`, and backs up replaced copies under
`~/.local/state/linux-config/backups/`. Full setup runs both steps.

Without the plugin, standard Zsh completion remains available. Ctrl+R depends on
the Debian fzf key-binding script; file previews require the Debian `bat` package.
Completion initializes before fzf-tab, autosuggestions and syntax highlighting.

## Git prompt

The prompt shows the current branch and working-tree status. `⇡10` means the
current branch has ten local commits ahead of its configured upstream, usually
`origin/main`. Committing increases that count; `git push` uploads the commits.
The count reflects the locally recorded upstream state; `git fetch` refreshes it.

`ga .` stages changes and `gc -m "message"` commits them through the project's
aliases. Neither command pushes or creates a branch. Project work stays on
`main` unless the user asks for a branch.

## Verification

The user confirmed Up/Down prefix search works. Interactive automated checks
cover repeated searches, exhaustion, direction changes, restoration of typed
input, multiword prefixes, empty-input browsing and Ctrl+R insertion in both
editing modes and both arrow formats. Installer fixtures check dry runs,
replacement backups, repeated installation and rejection of a mismatched commit.

The fzf-tab download failed in the restricted session because GitHub could not
be resolved. Live Tab selection, cancellation, previews and completion-based
grey suggestions still need user verification after installing the plugin.
