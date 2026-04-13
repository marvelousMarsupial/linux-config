"""Dev container (podman) management commands."""

import os
import platform
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional

from . import BIN_DIR, DEV_CONTAINER_PREFIX, RUNTIME_PACKAGES, STOW_DIR
from .common import command_exists, run


def get_arch_urls() -> dict:
    """Get architecture-specific download URLs."""
    machine = platform.machine()
    if machine in ("aarch64", "arm64"):
        return {
            "nvim": "nvim-linux-arm64.appimage",
            "yazi_zip": "yazi-aarch64-unknown-linux-gnu.zip",
            "yazi_dir": "yazi-aarch64-unknown-linux-gnu",
        }
    else:  # x86_64
        return {
            "nvim": "nvim-linux-x86_64.appimage",
            "yazi_zip": "yazi-x86_64-unknown-linux-gnu.zip",
            "yazi_dir": "yazi-x86_64-unknown-linux-gnu",
        }


def dev_container_name(name: str) -> str:
    """Get full container name with prefix."""
    if name.startswith(DEV_CONTAINER_PREFIX):
        return name
    return f"{DEV_CONTAINER_PREFIX}{name}"


def dev_container_exists(name: str) -> bool:
    """Check if a dev container exists."""
    full_name = dev_container_name(name)
    result = subprocess.run(
        ["podman", "container", "exists", full_name],
        capture_output=True
    )
    return result.returncode == 0


def dev_container_running(name: str) -> bool:
    """Check if a dev container is running."""
    full_name = dev_container_name(name)
    result = subprocess.run(
        ["podman", "inspect", "-f", "{{.State.Running}}", full_name],
        capture_output=True, text=True
    )
    return result.stdout.strip() == "true"


def cmd_dev_create(name: str, runtime: str, ports: Optional[list] = None,
                   volumes: Optional[list] = None) -> None:
    """Create a dev container with specified runtime."""
    if not command_exists("podman"):
        print("Error: podman not found.")
        sys.exit(1)

    full_name = dev_container_name(name)

    if dev_container_exists(name):
        print(f"Error: Container '{full_name}' already exists.")
        print(f"Use 'forge dev rm {name}' to remove it first.")
        sys.exit(1)

    print(f"==> Creating dev container '{full_name}' with runtime '{runtime}'...")

    # Build podman create command
    cmd = [
        "podman", "create",
        "--name", full_name,
        "--hostname", name,
        "-it",
    ]

    # Add port forwarding
    if ports:
        for port in ports:
            # Support both "8000" and "8000:8080" formats
            if ":" not in port:
                port = f"{port}:{port}"
            cmd.extend(["-p", port])

    # Add volume mounts
    if volumes:
        for vol in volumes:
            # Resolve relative paths to absolute
            if ":" in vol:
                host_path, container_path = vol.split(":", 1)
            else:
                host_path = vol
                container_path = "/workspace"
            host_path = str(Path(host_path).resolve())
            cmd.extend(["-v", f"{host_path}:{container_path}"])

    cmd.extend(["debian:trixie", "sleep", "infinity"])
    run(cmd)

    # Start the container
    run(["podman", "start", full_name])

    # Wait for container to be ready
    print("==> Waiting for container...", end="", flush=True)
    for _ in range(30):
        result = subprocess.run(
            ["podman", "exec", full_name, "cat", "/etc/os-release"],
            capture_output=True
        )
        if result.returncode == 0:
            print(" ready!")
            break
        print(".", end="", flush=True)
        time.sleep(0.5)
    else:
        print(" timeout!")
        sys.exit(1)

    # Install base packages
    print("==> Installing base packages...")
    base_pkgs = [
        "zsh", "tmux", "git", "curl", "wget", "unzip", "ripgrep", "bat",
        "ca-certificates", "gnupg", "locales", "trash-cli",
        "zsh-autosuggestions", "zsh-syntax-highlighting", "eza", "fzf",
        "file",  # needed for yazi MIME detection
        "ncurses-term",  # extended terminfo including foot terminal
        "make", "gcc",  # needed to compile telescope-fzf-native
        "python3-yaml", "python3-jinja2",  # needed for theme script
    ]
    run(["podman", "exec", full_name, "apt-get", "update", "-qq"])
    run(["podman", "exec", full_name, "apt-get", "install", "-y", "-qq"] + base_pkgs)

    # Create bat symlink (Debian installs as batcat)
    run(["podman", "exec", full_name, "ln", "-sf", "/usr/bin/batcat", "/usr/local/bin/bat"])

    # Generate locales
    run(["podman", "exec", full_name, "bash", "-c",
         "echo 'en_US.UTF-8 UTF-8' >> /etc/locale.gen && locale-gen"])

    # Install Node.js in all containers (needed for Mason LSP installs)
    print("==> Installing Node.js...")
    run(["podman", "exec", full_name, "bash", "-c",
         "curl -fsSL https://deb.nodesource.com/setup_lts.x | bash -"])
    run(["podman", "exec", full_name, "apt-get", "install", "-y", "-qq", "nodejs"])

    # Install runtime-specific packages
    if RUNTIME_PACKAGES.get(runtime):
        print(f"==> Installing {runtime} packages...")
        run(["podman", "exec", full_name, "apt-get", "install", "-y", "-qq"]
            + RUNTIME_PACKAGES[runtime])

    # Create python -> python3 symlink for python runtime
    if runtime == "python":
        run(["podman", "exec", full_name, "ln", "-sf",
             "/usr/bin/python3", "/usr/local/bin/python"])

    # Install bun for bun runtime
    if runtime == "bun":
        print("==> Installing bun...")
        run(["podman", "exec", full_name, "bash", "-c",
             "curl -fsSL https://bun.sh/install | bash"])

    # Install neovim (extract appimage - FUSE not available in containers)
    print("==> Installing neovim...")
    arch = get_arch_urls()
    run(["podman", "exec", full_name, "bash", "-c",
         "cd /tmp && "
         f"curl -fsSL https://github.com/neovim/neovim/releases/latest/download/{arch['nvim']} -o nvim.appimage && "
         "chmod +x nvim.appimage && "
         "./nvim.appimage --appimage-extract && "
         "mv squashfs-root /opt/nvim && "
         "rm nvim.appimage"])

    # Create nvim wrapper script with VIMRUNTIME set
    run(["podman", "exec", full_name, "bash", "-c",
         "cat > /usr/local/bin/nvim << 'EOF'\n"
         "#!/bin/bash\n"
         "export VIMRUNTIME=/opt/nvim/usr/share/nvim/runtime\n"
         "exec /opt/nvim/usr/bin/nvim \"$@\"\n"
         "EOF"])
    run(["podman", "exec", full_name, "chmod", "+x", "/usr/local/bin/nvim"])

    # Install starship
    print("==> Installing starship...")
    run(["podman", "exec", full_name, "bash", "-c",
         "curl -fsSL https://starship.rs/install.sh | sh -s -- -y"])

    # Install yazi (file manager) and ya (companion CLI for yazi.nvim)
    print("==> Installing yazi...")
    run(["podman", "exec", full_name, "bash", "-c",
         "cd /tmp && "
         f"curl -fsSL https://github.com/sxyazi/yazi/releases/latest/download/{arch['yazi_zip']} -o yazi.zip && "
         "unzip -q yazi.zip && "
         f"mv {arch['yazi_dir']}/yazi /usr/local/bin/ && "
         f"mv {arch['yazi_dir']}/ya /usr/local/bin/ && "
         f"rm -rf yazi.zip {arch['yazi_dir']}"])

    # Copy stow configs
    print("==> Copying dotfiles...")
    configs = [
        ("nvim/.config/nvim", "/root/.config/nvim"),
        ("zsh/.zshrc", "/root/.zshrc"),
        ("zsh/.zsh_aliases", "/root/.zsh_aliases"),
        ("tmux/.tmux.conf", "/root/.tmux.conf"),
        ("starship/.config/starship.toml", "/root/.config/starship.toml"),
        ("yazi/.config/yazi", "/root/.config/yazi"),
        ("theme/.config/theme", "/root/.config/theme"),
    ]

    # Create .config directory
    run(["podman", "exec", full_name, "mkdir", "-p", "/root/.config"])

    for src, dest in configs:
        src_path = STOW_DIR / src
        if src_path.exists():
            # Create parent directory if needed
            dest_dir = str(Path(dest).parent)
            run(["podman", "exec", full_name, "mkdir", "-p", dest_dir])
            run(["podman", "cp", str(src_path), f"{full_name}:{dest}"])

    # Copy theme script from bin/
    run(["podman", "exec", full_name, "mkdir", "-p", "/root/.local/bin"])
    run(["podman", "cp", str(BIN_DIR / "theme"), f"{full_name}:/root/.local/bin/theme"])

    # Set zsh as default shell
    run(["podman", "exec", full_name, "chsh", "-s", "/bin/zsh", "root"])

    # Add locale and terminal settings to prevent glitches (must be after copying .zshrc)
    run(["podman", "exec", full_name, "bash", "-c",
         "echo 'export LANG=en_US.UTF-8' >> /root/.zshrc && "
         "echo 'export LC_ALL=en_US.UTF-8' >> /root/.zshrc && "
         "echo 'export TERM=foot' >> /root/.zshrc &&"
         "echo 'export COLORTERM=truecolor' >> /root/.zshrc && "
         # Add theme script to PATH
         "echo 'export PATH=\"$HOME/.local/bin:$PATH\"' >> /root/.zshrc && "
         # Brighter autosuggestions color (default is too dark in containers)
         "echo 'export ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE=\"fg=244\"' >> /root/.zshrc && "
         # Terminal initialization for clean rendering
         "echo 'stty sane 2>/dev/null' >> /root/.zshrc && "
         "echo 'tput init 2>/dev/null' >> /root/.zshrc"])

    # Create directories needed by theme system
    run(["podman", "exec", full_name, "mkdir", "-p",
         "/root/.config/tmux", "/root/.cache/theme"])

    # Apply host's current theme if set
    host_theme_file = Path.home() / ".cache" / "theme" / "current"
    if host_theme_file.exists():
        current_theme = host_theme_file.read_text().strip()
        if current_theme:
            print(f"==> Applying theme '{current_theme}'...")
            run(["podman", "exec", full_name, "/root/.local/bin/theme", "apply", current_theme],
                check=False)

    # Add bun to PATH (must be after copying .zshrc)
    if runtime == "bun":
        run(["podman", "exec", full_name, "bash", "-c",
             "echo 'export BUN_INSTALL=\"$HOME/.bun\"' >> /root/.zshrc && "
             "echo 'export PATH=\"$BUN_INSTALL/bin:$PATH\"' >> /root/.zshrc"])

    # Create workspace directory
    run(["podman", "exec", full_name, "mkdir", "-p", "/workspace"])

    # Create vault directory for notes
    run(["podman", "exec", full_name, "mkdir", "-p", "/root/vault/daily"])

    # Pre-install nvim plugins and LSPs (so user doesn't see Mason errors)
    print("==> Installing nvim plugins and LSPs...")
    run(["podman", "exec", full_name, "bash", "-c",
         "nvim --headless '+Lazy! sync' +qa 2>/dev/null || true"])
    # Compile telescope-fzf-native (requires make/gcc)
    run(["podman", "exec", full_name, "bash", "-c",
         "cd /root/.local/share/nvim/lazy/telescope-fzf-native.nvim && make 2>/dev/null || true"])
    run(["podman", "exec", full_name, "bash", "-c",
         "nvim --headless '+MasonInstall lua-language-server typescript-language-server "
         "pyright tailwindcss-language-server vue-language-server clangd "
         "stylua prettier ruff shfmt' +qa 2>/dev/null || true"],
        check=False)
    # Give Mason a moment to finish
    time.sleep(2)

    print()
    print(f"==> Dev container '{full_name}' ready!")
    print(f"    Runtime:   {runtime}")
    print("    Workdir:   /workspace")
    if ports:
        print(f"    Ports:     {', '.join(ports)}")
    if volumes:
        print(f"    Volumes:   {', '.join(volumes)}")
    print()
    print(f"    Enter:  forge dev enter {name}")
    print(f"    Remove: forge dev rm {name}")


def cmd_dev_enter(name: str) -> None:
    """Enter a dev container."""
    full_name = dev_container_name(name)

    if not dev_container_exists(name):
        print(f"Error: Container '{full_name}' does not exist.")
        print(f"Create with: forge dev create {name} <runtime>")
        sys.exit(1)

    # Start if not running
    if not dev_container_running(name):
        print(f"==> Starting {full_name}...")
        run(["podman", "start", full_name])

    # Enter container (use /workspace as workdir if it exists)
    os.execlp("podman", "podman", "exec", "-it", "-w", "/workspace", full_name, "zsh")


def cmd_dev_list() -> None:
    """List dev containers."""
    result = subprocess.run(
        ["podman", "ps", "-a", "--filter", f"name={DEV_CONTAINER_PREFIX}",
         "--format", "table {{.Names}}\t{{.Status}}\t{{.Image}}"],
        capture_output=True, text=True
    )
    if result.stdout.strip():
        print(result.stdout)
    else:
        print("No dev containers found.")


def cmd_dev_rm(name: str, force: bool = False) -> None:
    """Remove a dev container."""
    full_name = dev_container_name(name)

    if not dev_container_exists(name):
        print(f"Error: Container '{full_name}' does not exist.")
        sys.exit(1)

    cmd = ["podman", "rm", full_name]
    if force:
        cmd.insert(2, "-f")

    print(f"==> Removing {full_name}...")
    run(cmd)
    print(f"==> Container '{full_name}' removed.")


def cmd_dev_stop(name: str) -> None:
    """Stop a dev container."""
    full_name = dev_container_name(name)

    if not dev_container_exists(name):
        print(f"Error: Container '{full_name}' does not exist.")
        sys.exit(1)

    print(f"==> Stopping {full_name}...")
    run(["podman", "stop", full_name])
    print(f"==> Container '{full_name}' stopped.")
