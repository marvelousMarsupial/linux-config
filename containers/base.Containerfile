# Base dev container: Debian trixie + the user's full shell experience.
#
# Goal: continuity. Drop into a `forge dev up myapp` shell and get the
# same zsh/tmux/nvim/yazi/starship setup as the host, with `uv` (Python),
# `fnm` (Node), Claude Code, and OpenCode already on PATH, and ssh-agent
# forwarded for `git push`. State (history, nvim plugins, project clones,
# git config, downloaded Python/Node versions, project venvs) lives in
# the `<name>-home` named volume mounted at /home/dev, so stop/start and
# container recreation all preserve the user's working set. See the
# header of `config/dev-container.toml` for the full persistence model.
#
# Designed for rootless podman with `--userns keep-id`. The `dev` user
# inside the container maps to the invoking host UID, so files written
# into the volume (or any bind mount) come back out owned by the host
# user with no chown dance.
#
# Dotfiles are *baked into* the image at build time (the COPY block near
# the bottom of this file). They are NOT bind-mounted from the host —
# the container is intentionally self-contained so the same image runs
# on a freshly-provisioned remote box without dragging the host fs along.
# Rebuild the image after editing dotfiles: `forge dev build`. New
# containers pick the changes up immediately; existing containers keep
# their volume's copy until you `podman volume rm <name>-home`.
#
# PID 1 is `sleep infinity` (set at `podman run` time); real shells come
# in via `podman exec -it … zsh`, which fires the .zshrc baked below
# (history, plugins, starship, tmux auto-attach, alias reminder).
#
# Build:  forge dev build
# Run:    forge dev up myapp

FROM docker.io/library/debian:trixie-slim

# ---------------------------------------------------------------------------
# System packages.
#
# Single layer with cleanup so the image stays small. Three groups:
#   1. Shell + editor + tooling the host's .zshrc / .zsh_aliases reference.
#      `zsh-autosuggestions` and `zsh-syntax-highlighting` are sourced
#      directly from /usr/share/ in .zshrc — without them the shell
#      errors on every prompt. `trash-cli` provides `trash-put`, which
#      .zsh_aliases binds to `rm`. `file` is the libmagic-based mime
#      detector: yazi shells out to `file --mime-type` for any path
#      whose extension doesn't disambiguate the type, and on a base
#      trixie-slim image the binary is absent, so opening yazi inside
#      the container greets the user with `Cannot find 'file' to
#      detect the file's mime type` on every directory listing. The
#      package is also a transitive dependency of countless build
#      tools (autoconf, debhelper, …) so it costs almost nothing to
#      pull explicitly here.
#   2. Build essentials + python3 (handy for ad-hoc scripts and `serve`
#      alias) + curl/git/ca-certs (needed for the install scripts below).
#   3. nodejs + npm: needed to install Claude Code globally and to run
#      JS dev tooling inside the container without an extra round-trip.
#   4. locales + ncurses-term: WITHOUT these, "look" inside the container
#      breaks badly compared to the host:
#        * `locales` + a generated en_US.UTF-8 locale teach glibc to
#          decode multi-byte UTF-8 sequences. The trixie-slim base
#          image ships only the C locale; without locale-gen below,
#          `LANG=en_US.UTF-8` (which the user's .zshrc exports)
#          silently falls back to C, multi-byte codepoints render as
#          `?`, and emojis / nerd-font glyphs / box-drawing characters
#          all look broken even though the host's foot terminal can
#          display them perfectly.
#        * `ncurses-term` ships the extended terminfo database that
#          includes `foot` and `foot-extra` (Debian trixie's
#          foot-terminfo apt package is now an empty transitional
#          shim; ncurses-term Provides: foot-terminfo). Without these
#          entries, ncurses applications (vim, yazi, less, htop) on a
#          host whose host-side terminal is foot fall back to
#          minimal/wrong escape sequences and render with bad colors,
#          missing styling, or visual artefacts. Host TERM propagates
#          straight into the container via `podman exec`, so the
#          container needs the matching terminfo on disk.
# ---------------------------------------------------------------------------
RUN apt-get update && apt-get install -y --no-install-recommends \
        zsh zsh-autosuggestions zsh-syntax-highlighting \
        tmux git git-delta curl ca-certificates \
        fzf ripgrep fd-find bat eza jq trash-cli file \
        build-essential python3 \
        unzip xz-utils \
        nodejs npm \
        locales ncurses-term tzdata \
        sudo \
    && rm -rf /var/lib/apt/lists/* \
    && ln -s /usr/bin/batcat /usr/local/bin/bat
# Notes on the apt list above:
#   * `git-delta` (binary `delta`) — installed from trixie main rather
#     than a curl-from-release because the package is in Debian and
#     we don't need to chase upstream. The user's git config can
#     opt in via `[core] pager = delta` and `[interactive] diffFilter
#     = delta --color-only`; we don't force a global config so this
#     stays an opt-in.
#   * `tzdata` ships /usr/share/zoneinfo + /etc/timezone defaults so
#     the host-bind of /etc/localtime + /etc/timezone (declared in
#     dev-container.toml's default_volumes.bind) has a tzdata layout
#     to layer on top of. Without tzdata, glibc time-formatting
#     functions fall back to UTC silently and timestamps inside the
#     container don't match the host's wall-clock perception of "now",
#     which surprises log-tailing and `git log --date=local`.
#   * `bat` apt package installs the binary as `/usr/bin/batcat` (not
#     `bat`) because Debian's old `bacula-console-bat` package
#     historically owned `/usr/bin/bat` (Debian bug #930299, won't-fix).
#     The upstream-recommended workaround — quoted verbatim in
#     `sharkdp/bat`'s README — is `ln -s /usr/bin/batcat
#     /usr/local/bin/bat`. We do that here so the container's
#     `cat = bat` zsh alias and any direct `bat` invocation work
#     out of the box. The host's `server` role mirrors this with a
#     post-cmd; the symlink lives in the same place on both.

# Generate the en_US.UTF-8 locale so the LANG / LC_ALL exports in the
# user's .zshrc actually resolve to a real locale instead of silently
# falling back to C. The /etc/locale.gen line for en_US.UTF-8 ships
# commented out by default; uncommenting + running locale-gen produces
# /usr/lib/locale/locale-archive entries that glibc consults. The ENV
# below makes the locale the container-wide default so even non-zsh
# entry points (sh -c …, podman exec without a login shell) get UTF-8.
RUN sed -i 's/^# *en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen \
    && locale-gen
ENV LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8

# Truecolor advertisement.
#
# foot is 24-bit-color from day one and the host's tmux is configured
# with `terminal-features ',foot*:RGB'` so 24-bit color escapes from
# inside tmux propagate untouched to foot. Inside the container, TUI
# apps need to *know* truecolor is available — most check `$COLORTERM`
# (nvim, opencode, lazygit, btop, bat) before falling back to
# 256-color quantisation. `podman exec -t` propagates TERM but NOT
# arbitrary env, so without this ENV line the container shell ends up
# with `COLORTERM` unset, apps quantise palettes to the 256-color
# cube, and catppuccin's purple/blue gradients render visibly off
# ("TUI graphics are wrong"). Setting it here gives every container
# a truecolor floor; ncurses-term's `tmux-256color` terminfo entry on
# trixie's ncurses 6.5 also advertises RGB via the Tc capability for
# the strict-terminfo apps.
ENV COLORTERM=truecolor

# ---------------------------------------------------------------------------
# Single-binary tools downloaded from upstream releases. Same pattern the
# host's `server` role uses for these — keeps the host and container on
# the same upstream binaries rather than diverging on Debian-packaged
# variants that may lag.
# ---------------------------------------------------------------------------

# Starship prompt.
RUN curl -fsSL https://starship.rs/install.sh | sh -s -- --yes

# Neovim — appimage extracted to /usr/local/nvim, symlinked onto PATH.
RUN curl -fsSL -o /tmp/nvim.appimage https://github.com/neovim/neovim/releases/latest/download/nvim-linux-x86_64.appimage \
    && chmod +x /tmp/nvim.appimage \
    && cd /tmp && ./nvim.appimage --appimage-extract >/dev/null 2>&1 \
    && mv /tmp/squashfs-root/usr /usr/local/nvim \
    && ln -s /usr/local/nvim/bin/nvim /usr/local/bin/nvim \
    && rm -rf /tmp/nvim.appimage /tmp/squashfs-root

# Yazi file manager.
RUN curl -fsSL -o /tmp/yazi.zip https://github.com/sxyazi/yazi/releases/latest/download/yazi-x86_64-unknown-linux-gnu.zip \
    && unzip -q /tmp/yazi.zip -d /tmp \
    && mv /tmp/yazi-x86_64-unknown-linux-gnu/yazi /usr/local/bin/yazi \
    && mv /tmp/yazi-x86_64-unknown-linux-gnu/ya /usr/local/bin/ya \
    && rm -rf /tmp/yazi.zip /tmp/yazi-*

# ---------------------------------------------------------------------------
# Toolchain managers — `uv` (Astral's Python toolchain manager: replaces
# pip/venv/pyenv/poetry) and `fnm` (Schniz/fnm: a Rust-based Node version
# manager, ~10× faster than nvm at shell startup).
#
# Both are installed to /usr/local/bin (image-managed, refreshed on
# rebuild, never shadowed by the home volume). The *data* they manage —
# downloaded Python interpreters, downloaded Node versions, project
# venvs — lands under $HOME (~/.local/share/uv, ~/.local/share/fnm) which
# is the persistent volume, so per-project toolchains survive
# `forge dev rm` and ride along into the next `forge dev up`.
#
# We pull from upstream's GitHub release artefacts directly rather than
# their `curl … | sh` install scripts so the install is reproducible at
# image-build time and doesn't depend on the script's $HOME-detection
# logic guessing right under root + Containerfile semantics.
# ---------------------------------------------------------------------------

# uv: tarball ships `uv` + `uvx` binaries.
RUN curl -fsSL -o /tmp/uv.tgz https://github.com/astral-sh/uv/releases/latest/download/uv-x86_64-unknown-linux-gnu.tar.gz \
    && tar -xzf /tmp/uv.tgz -C /tmp \
    && mv /tmp/uv-x86_64-unknown-linux-gnu/uv  /usr/local/bin/uv \
    && mv /tmp/uv-x86_64-unknown-linux-gnu/uvx /usr/local/bin/uvx \
    && rm -rf /tmp/uv.tgz /tmp/uv-x86_64-unknown-linux-gnu

# fnm: zip ships a single static binary.
RUN curl -fsSL -o /tmp/fnm.zip https://github.com/Schniz/fnm/releases/latest/download/fnm-linux.zip \
    && unzip -q /tmp/fnm.zip -d /tmp/fnm \
    && mv /tmp/fnm/fnm /usr/local/bin/fnm \
    && chmod +x /usr/local/bin/fnm \
    && rm -rf /tmp/fnm.zip /tmp/fnm

# lazygit — TUI git client, bound to the `lg` zsh alias. No apt package
# in trixie main (only available via Debian backports / unstable, both
# of which we avoid for reproducibility), so we pull the upstream static
# tarball — same pattern as nvim / yazi / uv / fnm above. Single static
# binary, ~10 MiB, no runtime deps. Pinned to `latest` because lazygit
# tags get released often and rolling at image-rebuild time matches the
# rest of the upstream-fetched tools' policy.
RUN curl -fsSL "https://github.com/jesseduffield/lazygit/releases/latest/download/lazygit_$(curl -fsSL https://api.github.com/repos/jesseduffield/lazygit/releases/latest | grep tag_name | cut -d '"' -f 4 | sed 's/^v//')_Linux_x86_64.tar.gz" -o /tmp/lazygit.tgz \
    && tar -xzf /tmp/lazygit.tgz -C /tmp lazygit \
    && mv /tmp/lazygit /usr/local/bin/lazygit \
    && chmod +x /usr/local/bin/lazygit \
    && rm -f /tmp/lazygit.tgz

# ---------------------------------------------------------------------------
# AI coding assistants — installed *system-wide* via npm so they land in
# /usr/local/lib/node_modules with shims in /usr/local/bin. We use npm
# rather than the upstream `curl … | sh` install scripts because the
# install scripts default to `$HOME/.opencode/bin` (or similar under
# `~/.local`), and when the Containerfile runs them as root the binary
# ends up in `/root/.opencode/bin/opencode` — off PATH for the `dev`
# user. The npm globals path is universally on PATH and image rebuilds
# refresh both packages atomically.
#
# Living in /usr/local means the named volume that mounts on /home/dev
# does NOT shadow them; if we put these under $HOME they'd vanish from
# any container whose volume already existed before the image rebuild.
#
# Claude Code: Anthropic's CLI (`claude`).
# OpenCode: sst/opencode AI coding agent (`opencode`).
# ---------------------------------------------------------------------------
RUN npm install -g @anthropic-ai/claude-code opencode-ai

# ---------------------------------------------------------------------------
# Host-only command stubs.
#
# Some commands the user reflexively types — `forge` is the obvious
# one — only make sense on the host (forge is the workstation
# provisioner; running it inside a dev container has no meaningful
# scope). Without a stub, typing `forge` in the container hits zsh's
# command_not_found_handler with a plain `command not found: forge`,
# which on Debian also runs the apt-index lookup and prints "no
# package found", which is not just unhelpful — it actively misleads
# (suggesting forge is something installable). The stub below
# intercepts the command, prints a one-line hint pointing at the
# host, and exits 127 so shell `&&` chains break correctly. Same
# pattern Gitpod uses for its `gp` command. The stub is mode 0755
# system-wide so it shadows any future PATH entry.
# ---------------------------------------------------------------------------
RUN printf '#!/bin/sh\necho "forge is a host-only command — exit this dev container to run it on the host." >&2\nexit 127\n' > /usr/local/bin/forge \
    && chmod 0755 /usr/local/bin/forge

# ---------------------------------------------------------------------------
# Clipboard inside the container — OSC52 shim named `wl-copy`.
#
# The host's wl-copy/wl-paste talk to the Wayland compositor's
# clipboard service, which doesn't exist inside a podman container.
# The right primitive here is OSC52: a terminal escape sequence
# (`ESC ] 52 ; c ; <base64-payload> ESC \`) that tells the *outer*
# terminal to put the decoded payload onto the host's clipboard.
# foot supports OSC52 since v1.4 (enabled by default), and the
# host's tmux is configured (in dotfiles/.tmux.conf) to advertise
# the `clipboard` capability for foot via `terminal-features
# ',foot*:clipboard'` plus `set-clipboard on` so the escape
# propagates from the inner pty up through tmux to foot without
# being stripped.
#
# This shim makes the dotfiles' `clip` zsh alias (which targets
# `wl-copy`) and any tmux copy-pipe binding that shells out to
# `wl-copy` work inside the container without any code change.
# Paste is intentionally NOT shimmed: OSC 52 read (`OSC 52 ; c ; ?`)
# is one-way-by-default in every modern terminal (foot included)
# for security reasons — programs running over a remote pty
# shouldn't be able to silently exfiltrate the user's clipboard.
# Use the terminal's native paste binding (Ctrl+Shift+V in foot)
# instead.
#
# Implementation note on quoting: heredocs inside a Containerfile
# RUN go through the shell's own parsing, which means dollar signs
# in the script body need careful escaping. We dodge that by using
# a single `printf` with the script body as the format string, no
# variable interpolation needed inside the script — the only
# runtime `$()` is the base64 invocation, which lives unescaped
# inside the printf format string.
# ---------------------------------------------------------------------------
RUN printf '#!/bin/sh\n# OSC52 clipboard shim — emits the escape sequence the outer terminal\n# reads to place stdin onto the host clipboard. Paired with foot v1.4+\n# (osc52 enabled by default) and tmux `set-clipboard on` +\n# `terminal-features ,foot*:clipboard`. See the Containerfile block\n# comment for the full rationale.\nprintf "\\033]52;c;%%s\\033\\\\" "$(base64 -w0)"\n' > /usr/local/bin/wl-copy \
    && chmod 0755 /usr/local/bin/wl-copy

# ---------------------------------------------------------------------------
# Non-root user.
#
# UID 1000 matches the typical first non-system user on Debian; rootless
# podman's `--userns keep-id` (set by `forge dev up`) re-maps the host UID
# onto this user at runtime regardless. Passwordless sudo is intentional —
# the container is a developer workstation, not a hardened production box.
# ---------------------------------------------------------------------------
RUN useradd -m -s /bin/zsh -u 1000 dev \
    && echo 'dev ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/dev \
    && mkdir -p /run/user/1000 \
    && chown dev:dev /run/user/1000 \
    && chmod 700 /run/user/1000
# Pre-create $XDG_RUNTIME_DIR for the dev user.
#
# On a host, systemd-logind creates `/run/user/$UID` as a tmpfs at
# login — that's where systemd, dbus user session, fnm, gpg-agent,
# and a long tail of other tools write their per-session runtime
# state. Inside a podman container there's no logind, so this
# path doesn't exist by default, and any tool that consults
# `$XDG_RUNTIME_DIR` (or hardcodes `/run/user/$UID`) fails on
# first use.
#
# The visible failure mode that surfaced this was fnm:
#
#     error: Can't create the symlink for multishells at
#     "/run/user/1000/fnm_multishells/<pid>_<ts>". Maybe there are
#     some issues with permissions for the directory? No such file
#     or directory (os error 2)
#
# Creating the directory at image-build time, owned by the dev
# user with mode 0700 (matching what logind creates on a host),
# fixes fnm and every other XDG_RUNTIME_DIR consumer at once.
# Mode 0700 is the convention — many tools refuse to use the dir
# if it's group/world-readable since it can hold session secrets.
#
# Note: this directory only persists across container restart if
# podman doesn't tmpfs-mount /run. Rootless podman without
# `--systemd` (which is our setup — `forge dev up` doesn't pass
# --systemd) leaves /run as a regular directory layered over the
# image, so the dir we create here survives. If a future change
# adds --systemd or mounts /run as tmpfs, this directory will
# vanish per-run and we'll need a runtime hook (entrypoint or
# zshrc) to recreate it.

USER dev
WORKDIR /home/dev

# ---------------------------------------------------------------------------
# Bake the host's dotfiles into the image.
#
# These are scoped deliberately: shell rc files, tmux config, and the
# editor / file-manager / prompt / theme configs from .config/. We
# explicitly do NOT copy `.local/bin/` or sway/foot/fuzzel/kanshi — those
# are desktop-role scripts and compositor configs that have no meaning
# inside a headless container, and pulling them in would either error out
# (missing wayland deps) or pollute PATH with broken commands.
#
# `--chown=dev:dev` so the files come out owned by the runtime user.
# These paths are inside /home/dev, which is also the named-volume mount
# point — see the persistence note at the top of dev-images.toml for
# how podman seeds the volume from these baked-in files on first run.
# ---------------------------------------------------------------------------
COPY --chown=dev:dev dotfiles/.zshrc                /home/dev/.zshrc
COPY --chown=dev:dev dotfiles/.zsh_aliases          /home/dev/.zsh_aliases
COPY --chown=dev:dev dotfiles/.zsh_alias_reminder   /home/dev/.zsh_alias_reminder
COPY --chown=dev:dev dotfiles/.tmux.conf            /home/dev/.tmux.conf
COPY --chown=dev:dev dotfiles/.bashrc               /home/dev/.bashrc
COPY --chown=dev:dev dotfiles/.config/nvim          /home/dev/.config/nvim
COPY --chown=dev:dev dotfiles/.config/yazi          /home/dev/.config/yazi
COPY --chown=dev:dev dotfiles/.config/starship.toml /home/dev/.config/starship.toml
COPY --chown=dev:dev dotfiles/.config/environment.d /home/dev/.config/environment.d
# Note: dotfiles/.config/theme/ is deliberately NOT copied. That tree
# holds the *templates* and *schemes* that the host's `forge theme
# apply` consumes to render per-target color configs. Inside a dev
# container `forge` isn't on PATH (and is intentionally stubbed to
# refuse to run, see /usr/local/bin/forge above), so the templates
# would never be rendered — they'd just sit as 100 KiB of inert
# .j2 + .yaml in /home/dev/.config/theme/. The *rendered* outputs
# that the container actually consumes — `nvim/lua/plugins/colorscheme.lua`,
# `yazi/theme.toml`, `tmux/colors.conf` — are baked in as part of
# the explicit dotfile COPYs above, frozen at image-build time. To
# update the container's theme, run `forge theme apply <scheme>` on
# the host, then `forge dev build` to bake the new rendered files
# into a fresh image, then `podman volume rm <name>-home` so the
# next `forge dev up` re-seeds /home/dev from the rebuilt image
# (the named volume's first-run seed is what makes existing
# containers stale until the volume is wiped).

# Surface the container's bare name to starship's `container` module so
# the prompt shows which dev container the shell is in.
ENV CONTAINER_NAME=forge-dev
