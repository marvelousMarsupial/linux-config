# Dev Services

Browser-based development tools running in [[podman]] containers. Available in `dev` role.

## Overview

| Service | Port | Purpose |
|---------|------|---------|
| Coder | 8000 | VS Code in browser |
| Jupyter | 9000 | JupyterLab notebooks |
| CloudBeaver | 8978 | Database manager |

All bind to localhost only. Access via SSH tunnel or local machine.

## Access Pattern

### Remote Machine

```bash
# On local machine: SSH tunnel
ssh -L 8000:localhost:8000 -L 9000:localhost:9000 user@server

# On server: start service
coder ~/project

# In browser: http://localhost:8000
```

### Local Machine

```bash
# Start service
coder ~/project

# Open: http://localhost:8000
```

## Coder

VS Code in browser via code-server.

### Commands

```bash
coder              # Open current directory
coder ~/project    # Open specific directory
coder -h           # Help
coder-stop         # Stop container
```

### Details

| Property | Value |
|----------|-------|
| Port | 8000 |
| Image | docker.io/codercom/code-server:latest |
| Extensions | Persist in container |
| Settings | Persist in container |

### Features

- Full VS Code experience
- Terminal access
- Extension marketplace
- Git integration
- Remote file access

### First Run

1. Start coder
2. Open http://localhost:8000
3. Password shown in terminal output

## Jupyter

JupyterLab with scientific Python stack.

### Commands

```bash
jupyter            # Use ~/notebooks directory
jupyter ~/data     # Use specific directory
jupyter -h         # Help
jupyter-stop       # Stop container
```

### Details

| Property | Value |
|----------|-------|
| Port | 9000 |
| Image | docker.io/jupyter/scipy-notebook:latest |
| Python | 3.x with scipy stack |
| Data | Mounted from specified directory |

### Included Libraries

- numpy, scipy, pandas
- matplotlib, seaborn
- scikit-learn
- And many more (scipy-notebook)

### First Run

1. Start jupyter
2. Token printed in terminal
3. Open http://localhost:9000?token=XXX

## CloudBeaver

Web-based database manager (DBeaver in browser).

### Commands

```bash
cloudbeaver        # Start
cloudbeaver -h     # Help
cloudbeaver-stop   # Stop container
```

### Details

| Property | Value |
|----------|-------|
| Port | 8978 |
| Image | docker.io/dbeaver/cloudbeaver:latest |
| Data | ~/.cloudbeaver/workspace |

### Supported Databases

- PostgreSQL
- MySQL/MariaDB
- SQLite
- MongoDB
- And many more

### First Run

1. Start cloudbeaver
2. Open http://localhost:8978
3. Complete first-run wizard
4. Add database connections

## Container Management

### Viewing Status

```bash
podman ps                   # Running containers
podman ps -a                # All containers
```

### Logs

```bash
podman logs coder
podman logs jupyter
podman logs cloudbeaver
```

### Stopping All

```bash
podman stop --all
```

### Removing

```bash
podman rm coder
podman rm jupyter
podman rm cloudbeaver
```

## vs Dev Containers

| Feature | Dev Services | [[dev-containers]] |
|---------|--------------|-------------------|
| Purpose | Single-purpose tools | Full dev environment |
| Interface | Web browser | Terminal (zsh) |
| Your dotfiles | No | Yes |
| Shell access | Limited | Full |
| Use case | Quick editing, notebooks | Project development |

## Persistence

### Coder

VS Code settings and extensions persist in container. Remove container to reset:
```bash
podman rm coder
```

### Jupyter

Notebooks saved to mounted directory (~/notebooks by default). Container can be removed without losing work.

### CloudBeaver

Workspace saved to ~/.cloudbeaver/workspace. Database connection configs persist there.

## Network Access

Services bind to 127.0.0.1 (localhost only) for security.

### Accessing External Services

Containers can reach:
- Host machine: `host.containers.internal`
- Internet: Normal access

### Database Connection Example

In CloudBeaver, connect to PostgreSQL on host:
```
Host: host.containers.internal
Port: 5432
```

## Troubleshooting

### Service Won't Start

```bash
podman logs <service>
podman rm <service>  # Then restart
```

### Port Already In Use

```bash
ss -tlnp | grep 8000  # Check what's using port
```

### Can't Access From Browser

1. Check service is running: `podman ps`
2. Check SSH tunnel is active (if remote)
3. Try http://127.0.0.1:PORT instead of localhost

### Permission Issues

Check volume mount:
```bash
podman inspect coder --format '{{.Mounts}}'
```

## Files

| Path | Purpose |
|------|---------|
| `bin/coder` | Coder start script |
| `bin/jupyter` | Jupyter start script |
| `bin/cloudbeaver` | CloudBeaver start script |
| `~/.cloudbeaver/workspace` | CloudBeaver data |

See also: [[podman]], [[dev-containers]], [[roles]]
