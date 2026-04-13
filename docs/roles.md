# Roles

Ansible roles for workstation provisioning. Each role inherits from its parent.

## Hierarchy

```
server (base CLI)
├── dev           → + dev services, build tools
├── desktop       → + Sway GUI, flatpaks
│   ├── remote-desktop → + VNC/Sunshine
│   └── attack-box     → + CTF/pentesting (via remote-desktop, no firewall)
└── net-box       → + network analysis
```

## Server (base)

Base CLI environment. Foundation for all other roles.

### Packages

| Category | Packages |
|----------|----------|
| Shell | zsh, tmux, bash-completion |
| Files | eza, bat, fd-find, ripgrep, yazi, tree |
| System | btop, htop, curl, wget, unzip |
| Dev | git, stow |
| Containers | podman, incus |

### Tools Installed

| Tool | Source | Description |
|------|--------|-------------|
| nvim | GitHub AppImage | Neovim 0.11+ |
| starship | curl installer | Cross-shell prompt |
| yazi | GitHub release | Terminal file manager |
| fzf | apt | Fuzzy finder |

### Stow Packages

Dotfiles symlinked: `bash`, `zsh`, `tmux`, `nvim`, `yazi`, `starship`, `theme`

### Theme System

Python script + Jinja2 templates applying Base16 colors across all apps.

See [[theme]] for details.

## Dev

Server + development tools and services.

### Additional Packages

| Category | Packages |
|----------|----------|
| Build | build-essential, cmake, clang, gcc, g++ |
| Python | python3, python3-pip, python3-venv, ipython |
| Node | nodejs (LTS via nodesource) |
| Go | golang |

### Dev Tools

| Tool | Source | Description |
|------|--------|-------------|
| lazygit | GitHub release | Git TUI |
| gh | apt | GitHub CLI |
| tea | GitHub release | Gitea CLI |

### Dev Services (podman)

On-demand containers for browser-based tools:

| Service | Port | Command |
|---------|------|---------|
| Coder | 8000 | `coder [project-dir]` |
| JupyterLab | 9000 | `jupyter [notebook-dir]` |
| CloudBeaver | 8978 | `cloudbeaver` |

Each service has a stop command: `coder-stop`, `jupyter-stop`, etc.

See [[dev-services]] for details.

### Dev Containers

Isolated development environments via [[forge]]:

```bash
forge dev create myapp python
forge dev enter myapp
```

See [[dev-containers]] for details.

## Desktop

Server + Sway Wayland GUI.

### Additional Packages

| Category | Packages |
|----------|----------|
| WM | sway, swaylock, swayidle, swaybg |
| Terminal | foot |
| Launcher | fuzzel |
| Notifications | mako |
| Screenshot | grim, slurp |
| Clipboard | wl-clipboard, cliphist |
| Audio | pipewire, wireplumber, pavucontrol |
| Bluetooth | bluez, blueman |
| Files | udiskie (USB automount) |
| Brightness | brightnessctl |
| Player | playerctl |

### Flatpaks

| App | Description |
|-----|-------------|
| Obsidian | Note-taking (markdown) |
| Zen Browser | Firefox-based browser |

### Stow Packages (added)

`sway`, `foot`, `fuzzel`, `gtk-3.0`, `gtk-4.0`, `desktop-entries`

### Scripts

All scripts in `bin/` (added to PATH):

| Script | Description |
|--------|-------------|
| launcher | Fuzzel wrapper with flatpak support |
| power-menu | Lock/suspend/logout/reboot/shutdown |
| status-panel | System info popup ($mod+i) |
| wifi-menu | WiFi selector ($mod+w) |
| battery-monitor | Low battery warnings |
| keybinds | Keybinding reference |
| notes-toggle | Toggle notes window ($mod+n) |
| notes-picker | Search notes ($mod+Shift+n) |

### Autostart

Started by Sway config:

- mako (notifications)
- udiskie --tray (USB automount)
- wl-paste --watch cliphist store (clipboard history)
- swayidle (lock on sleep)
- battery-monitor (warnings)

## Remote Desktop

Desktop + headless GUI access for remote machines.

### Additional Packages

| Tool | Description |
|------|-------------|
| wayvnc | VNC server for Wayland |
| Sunshine | Moonlight game streaming server |

### Scripts

Scripts in `bin/` (added to PATH):

| Script | Alias | Description |
|--------|-------|-------------|
| remote-start | rstart | Start headless Sway + wayvnc + mako |
| remote-stop | rstop | Stop all remote desktop services |

### Usage

```bash
# On remote machine
rstart

# On local machine
# Connect via VNC client to remote:5900
# Or use Moonlight for Sunshine
```

## Net-Box

Server + CLI network monitoring. Blue team.

### Traffic Capture
tshark, tcpdump, termshark, tcpflow, ngrep

### Bandwidth Monitoring
iftop, nethogs, vnstat, bmon, iperf3

### Network Discovery
nmap, arp-scan, netdiscover

### IDS
suricata (you configure rules)

### Utilities
netcat, socat, mtr, dnsutils, whois

See [[net-box]] for usage guide.

## Attack-Box

Remote-desktop + CTF/pentesting tools. GUI access via VNC for Wireshark/Burp.

**Key differences from desktop:**
- No firewall (all ports open for reverse shells)
- No personal apps (Element, Obsidian, Zen)
- Pentesting tools added

### Inheritance

```
server → desktop → remote-desktop → attack-box
```

### Browsers

| Browser | Purpose |
|---------|---------|
| firefox-esr | Proxy through Burp |
| chromium | Alternate browser |

### Network Analysis

| Tool | Description |
|------|-------------|
| wireshark | GUI packet capture |
| tshark | CLI packet capture |
| nmap | Port scanning |
| masscan | Fast scanning |
| tcpdump | Low-level capture |

### Web Testing

| Tool | Description |
|------|-------------|
| burpsuite | Web proxy/scanner |
| sqlmap | SQL injection |
| nikto | Web scanner |
| gobuster | Directory brute |
| wfuzz | Web fuzzer |

### Password Cracking

| Tool | Description |
|------|-------------|
| hashcat | GPU cracking |
| john | John the Ripper |
| hydra | Online brute force |

### Python Tools

ipython3, pwntools, scapy, impacket, requests, beautifulsoup4

### Wordlists

`/usr/share/seclists/`, `/usr/share/wordlists/`

See [[attack-box]] for full details.

## Role Selection

### Choosing a Role

| Use Case | Role |
|----------|------|
| Headless server | server |
| Development workstation (no GUI) | dev |
| Daily driver with GUI | desktop |
| Remote access needed | remote-desktop |
| Network troubleshooting | net-box |
| Security testing/CTF | attack-box |

### Switching Roles

```bash
forge switch desktop    # Intelligent: only changes what's different
```

See [[forge]] for commands.

## Files

| Path | Purpose |
|------|---------|
| `ansible/roles/<role>/tasks/main.yml` | Role entry point |
| `ansible/roles/<role>/tasks/packages.yml` | Package installation |
| `ansible/roles/<role>/tasks/dotfiles.yml` | Stow symlinks |
| `ansible/roles/<role>/defaults/main.yml` | Default variables |
| `~/.cache/forge/role` | Current role cache |

## Architecture Support

All roles support both x86_64 and ARM64 (aarch64). Architecture-specific variables in `ansible/group_vars/all.yml`:

| Variable | x86_64 | ARM64 |
|----------|--------|-------|
| arch_golang | amd64 | arm64 |
| arch_rust | x86_64-unknown-linux-gnu | aarch64-unknown-linux-gnu |
| arch_deb | amd64 | arm64 |
| arch_nvim | x86_64 | arm64 |

See also: [[forge]], [[stow]], [[theme]]
