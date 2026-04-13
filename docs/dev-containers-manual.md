# Dev Containers (Manual)

How to create dev containers manually with Dockerfiles and podman, without [[forge]].

## Quick Start

```bash
# Build image
podman build -t dev-python -f Dockerfile.python .

# Run container
podman run -it --name myapp -w /workspace dev-python
```

## Base Dockerfile

All runtimes share this base. Save as `Dockerfile.base`:

```dockerfile
FROM debian:trixie

# Prevent interactive prompts
ENV DEBIAN_FRONTEND=noninteractive

# Base packages
RUN apt-get update && apt-get install -y \
    zsh tmux git curl wget unzip \
    ripgrep bat eza fzf file \
    ca-certificates gnupg locales \
    trash-cli zsh-autosuggestions zsh-syntax-highlighting \
    && rm -rf /var/lib/apt/lists/*

# Locale
RUN echo 'en_US.UTF-8 UTF-8' >> /etc/locale.gen && locale-gen
ENV LANG=en_US.UTF-8

# bat symlink (Debian uses batcat)
RUN ln -sf /usr/bin/batcat /usr/local/bin/bat

# Node.js (for Mason LSP installs)
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Neovim (extract AppImage - no FUSE in containers)
RUN cd /tmp \
    && curl -fsSL https://github.com/neovim/neovim/releases/latest/download/nvim-linux-x86_64.appimage -o nvim.appimage \
    && chmod +x nvim.appimage \
    && ./nvim.appimage --appimage-extract \
    && mv squashfs-root /opt/nvim \
    && rm nvim.appimage

# Neovim wrapper (sets VIMRUNTIME)
RUN echo '#!/bin/bash\nexport VIMRUNTIME=/opt/nvim/usr/share/nvim/runtime\nexec /opt/nvim/usr/bin/nvim "$@"' > /usr/local/bin/nvim \
    && chmod +x /usr/local/bin/nvim

# Starship prompt
RUN curl -fsSL https://starship.rs/install.sh | sh -s -- -y

# Yazi + ya companion
RUN cd /tmp \
    && curl -fsSL https://github.com/sxyazi/yazi/releases/latest/download/yazi-x86_64-unknown-linux-gnu.zip -o yazi.zip \
    && unzip -q yazi.zip \
    && mv yazi-x86_64-unknown-linux-gnu/yazi /usr/local/bin/ \
    && mv yazi-x86_64-unknown-linux-gnu/ya /usr/local/bin/ \
    && rm -rf yazi.zip yazi-x86_64-unknown-linux-gnu

# Default shell
RUN chsh -s /bin/zsh root

# Workspace
WORKDIR /workspace

CMD ["zsh"]
```

## Runtime Dockerfiles

### Python

```dockerfile
FROM dev-base

RUN apt-get update && apt-get install -y \
    python3 python3-pip python3-venv ipython3 \
    && rm -rf /var/lib/apt/lists/*

RUN ln -sf /usr/bin/python3 /usr/local/bin/python
```

### C/C++

```dockerfile
FROM dev-base

RUN apt-get update && apt-get install -y \
    gcc g++ gdb make cmake clang \
    && rm -rf /var/lib/apt/lists/*
```

### TypeScript/Node

```dockerfile
FROM dev-base

# Node.js already installed in base
# Add any additional tools here
```

### Nuxt/Vue

```dockerfile
FROM dev-base

# Node.js already installed in base
# Nuxt will be installed per-project via npx
```

### Bun

```dockerfile
FROM dev-base

RUN curl -fsSL https://bun.sh/install | bash
ENV BUN_INSTALL="/root/.bun"
ENV PATH="$BUN_INSTALL/bin:$PATH"
```

### Java

```dockerfile
FROM dev-base

RUN apt-get update && apt-get install -y \
    openjdk-17-jdk maven gradle \
    && rm -rf /var/lib/apt/lists/*

ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64
```

## Building Images

```bash
# Build base first
podman build -t dev-base -f Dockerfile.base .

# Build runtime images
podman build -t dev-python -f Dockerfile.python .
podman build -t dev-c -f Dockerfile.c .
podman build -t dev-ts -f Dockerfile.ts .
```

## Running Containers

### Basic (self-contained)

```bash
podman run -it --name myapp dev-python
```

### With port forwarding

```bash
podman run -it --name myapp -p 8000:8000 dev-python
```

### With volume mount

```bash
podman run -it --name myapp -v .:/workspace dev-python
```

### Detached (background)

```bash
podman run -d --name myapp dev-python sleep infinity
podman exec -it -w /workspace myapp zsh
```

## Adding Dotfiles

### Option 1: COPY in Dockerfile

```dockerfile
FROM dev-base

COPY nvim /root/.config/nvim
COPY .zshrc /root/.zshrc
COPY .tmux.conf /root/.tmux.conf
```

### Option 2: Mount at runtime

```bash
podman run -it \
    -v ~/.config/nvim:/root/.config/nvim:ro \
    -v ~/.zshrc:/root/.zshrc:ro \
    -v ~/.tmux.conf:/root/.tmux.conf:ro \
    dev-python
```

### Option 3: Copy after create

```bash
podman create --name myapp dev-python sleep infinity
podman cp ~/.config/nvim myapp:/root/.config/nvim
podman cp ~/.zshrc myapp:/root/.zshrc
podman start myapp
podman exec -it myapp zsh
```

## Container Management

```bash
# List
podman ps -a

# Start stopped container
podman start myapp

# Enter running container
podman exec -it -w /workspace myapp zsh

# Stop
podman stop myapp

# Remove
podman rm myapp

# Remove with force (stops first)
podman rm -f myapp
```

## Minimal Dockerfile

If you want something simpler:

```dockerfile
FROM debian:trixie

RUN apt-get update && apt-get install -y \
    zsh git curl neovim python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
CMD ["zsh"]
```

Build and run:
```bash
podman build -t dev-minimal .
podman run -it --name myapp dev-minimal
```

## Multi-Stage for Smaller Images

```dockerfile
# Build stage
FROM debian:trixie AS builder
RUN apt-get update && apt-get install -y curl unzip
RUN curl -fsSL https://github.com/sxyazi/yazi/releases/latest/download/yazi-x86_64-unknown-linux-gnu.zip -o yazi.zip \
    && unzip yazi.zip

# Final stage
FROM debian:trixie
COPY --from=builder /yazi-x86_64-unknown-linux-gnu/yazi /usr/local/bin/
# ... rest of setup
```

## Compose File

For multiple containers (e.g., app + database):

```yaml
# podman-compose.yml
version: "3"
services:
  app:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - .:/workspace
    depends_on:
      - db
    command: sleep infinity

  db:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: dev
    ports:
      - "5432:5432"
```

Run:
```bash
podman-compose up -d
podman-compose exec app zsh
```

## Tips

### Keep Images Small

- Use `--no-install-recommends` with apt
- Clean apt cache: `rm -rf /var/lib/apt/lists/*`
- Combine RUN commands to reduce layers

### Persist Data

For data you want to keep between container rebuilds:
```bash
podman volume create myapp-data
podman run -v myapp-data:/workspace/data dev-python
```

### ARM64 Support

For ARM machines, change AppImage URL:
```dockerfile
# x86_64
curl -fsSL https://github.com/neovim/neovim/releases/latest/download/nvim-linux-x86_64.appimage

# ARM64
curl -fsSL https://github.com/neovim/neovim/releases/latest/download/nvim-linux-arm64.appimage
```

## vs Forge Dev Containers

| Aspect | Manual | Forge |
|--------|--------|-------|
| Flexibility | Full control | Preset runtimes |
| Speed | Slow (build each time) | Fast (builds on create) |
| Dotfiles | Manual copy | Auto from stow/ |
| Reproducible | Yes (Dockerfile) | Yes (code in dev.py) |

Use manual when you need custom setup. Use forge for quick standard environments.

## Files

Save Dockerfiles in `~/linux-config/dockerfiles/` or wherever convenient.

See also: [[dev-containers]], [[podman]], [[forge]]
