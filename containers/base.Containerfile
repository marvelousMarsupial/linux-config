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
#      .zsh_aliases binds to `rm`.
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
        tmux git curl ca-certificates \
        fzf ripgrep fd-find bat eza jq trash-cli \
        build-essential python3 \
        unzip xz-utils \
        nodejs npm \
        locales ncurses-term \
        sudo \
    && rm -rf /var/lib/apt/lists/*

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
# Non-root user.
#
# UID 1000 matches the typical first non-system user on Debian; rootless
# podman's `--userns keep-id` (set by `forge dev up`) re-maps the host UID
# onto this user at runtime regardless. Passwordless sudo is intentional —
# the container is a developer workstation, not a hardened production box.
# ---------------------------------------------------------------------------
RUN useradd -m -s /bin/zsh -u 1000 dev \
    && echo 'dev ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/dev

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
COPY --chown=dev:dev dotfiles/.config/theme         /home/dev/.config/theme
COPY --chown=dev:dev dotfiles/.config/environment.d /home/dev/.config/environment.d

# Surface the container's bare name to starship's `container` module so
# the prompt shows which dev container the shell is in.
ENV CONTAINER_NAME=forge-dev
