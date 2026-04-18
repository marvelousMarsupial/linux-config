# Base dev container: Debian trixie with zsh, tmux, neovim, starship, yazi.
#
# Designed for rootless podman with `--userns keep-id`. The `dev` user maps
# to the host UID so bind-mounted files have correct ownership. PID 1 is
# `sleep infinity` (set at `podman run` time), real shells come in via
# `podman exec`.
#
# The host's dotfiles are bind-mounted read-only at runtime — this image
# only installs the tools, not the configs. Theme inherits from the host
# via the mounted `~/.config/theme/`.
#
# Build:  forge dev build base
# Run:    forge dev up myapp

FROM docker.io/library/debian:trixie-slim

# Single layer: system packages + cleanup in one RUN to keep image small.
RUN apt-get update && apt-get install -y --no-install-recommends \
        zsh tmux git curl ca-certificates \
        fzf ripgrep fd-find bat eza jq \
        build-essential python3 \
        unzip xz-utils \
    && rm -rf /var/lib/apt/lists/*

# Starship prompt — single static binary.
RUN curl -fsSL https://starship.rs/install.sh | sh -s -- --yes

# Neovim — appimage extracted to /usr/local/bin.
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
    && rm -rf /tmp/yazi.zip /tmp/yazi-*

# Non-root user. UID 1000 matches most host users; `--userns keep-id`
# remaps at runtime anyway. Passwordless sudo for package installs inside.
RUN useradd -m -s /bin/zsh -u 1000 dev \
    && apt-get update && apt-get install -y --no-install-recommends sudo \
    && echo 'dev ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/dev \
    && rm -rf /var/lib/apt/lists/*

USER dev
WORKDIR /home/dev

# Prompt shows container name via starship's `container` module.
ENV CONTAINER_NAME=forge-dev
