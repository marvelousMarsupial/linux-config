# Forge

CLI tool for workstation provisioning. Python script at `bin/forge`.

## Commands Overview

| Command | Description |
|---------|-------------|
| `forge setup <role>` | Install role |
| `forge teardown [role]` | Remove role |
| `forge switch <role>` | Change roles |
| `forge status` | Show current state |
| `forge theme <args>` | Theme passthrough |
| `forge cleanup` | Remove ansible |
| `forge lxc <role>` | Create LXC container |
| `forge vm <role>` | Create VM |
| `forge instances` | List incus instances |
| `forge delete <name>` | Delete instance |
| `forge dev <cmd>` | Dev containers |

## Role Commands

### setup
```bash
forge setup server         # Base CLI
forge setup dev            # + dev services
forge setup desktop        # + Sway GUI
forge setup remote-desktop # + VNC/Sunshine
forge setup net-box        # + network tools
forge setup attack-box     # + CTF tools
```

Shorthand: `forge server` = `forge setup server`

### teardown
```bash
forge teardown             # Current role only
forge teardown desktop     # Specific role
forge teardown -f          # Full teardown (remove packages)
```

### switch
```bash
forge switch dev           # Teardown current, setup dev
```

Intelligent: only tears down components not in new role.

### status
```bash
forge status
# Role: desktop
# Theme: catppuccin-mocha
```

### cleanup
```bash
forge cleanup              # Remove ansible after setup
```

## Theme Commands

Passthrough to `theme` script:

```bash
forge theme list           # Available schemes
forge theme apply gruvbox-dark-hard
forge theme current        # Show palette
```

See [[theme]] for details.

## Incus Commands

Create provisioned VMs/containers:

```bash
# LXC container
forge lxc server                    # Creates server-lxc
forge lxc attack-box -n mybox       # Custom name

# VM
forge vm dev                        # Creates dev-vm
forge vm net-box -n scanner         # Custom name

# List
forge instances

# Delete
forge delete server-lxc
forge delete mybox -f               # Force (stop first)
```

Instance name becomes username and hostname inside.

See [[incus]] for more.

## Dev Container Commands

Isolated dev environments via podman:

```bash
# Create
forge dev create myapp python
forge dev create myapp python -p 8000 -v .:/workspace

# Enter
forge dev enter myapp

# Manage
forge dev list
forge dev stop myapp
forge dev rm myapp
forge dev rm myapp -f               # Force
```

Runtimes: `python`, `c`, `ts`, `nuxt`, `bun`, `java`, `base`

See [[dev-containers]] for details.

## Files

| Path | Purpose |
|------|---------|
| `bin/forge` | Main CLI (argparse + dispatch) |
| `bin/forge_lib/__init__.py` | Constants, Role enum |
| `bin/forge_lib/common.py` | Utilities |
| `bin/forge_lib/ansible.py` | Role commands |
| `bin/forge_lib/incus.py` | Incus commands |
| `bin/forge_lib/dev.py` | Dev container commands |
| `~/.cache/forge/role` | Current role cache |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (missing dependency, container exists, etc) |

## Environment

Requires:
- Python 3.10+
- ansible-playbook (auto-installed)
- podman (for dev containers)
- incus (for VMs/LXC)

See also: [[roles]], [[dev-containers]], [[incus]]
