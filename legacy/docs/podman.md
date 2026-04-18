# Podman

Rootless container runtime. Docker-compatible but runs as user without daemon.

## Requirements

Rootless podman requires these packages (installed by server role):
- `podman` - container runtime
- `uidmap` - user namespace mapping for rootless containers
- `passt` - network namespace tool (provides `pasta` for networking)

## Why Podman

| Feature | Podman | Docker |
|---------|--------|--------|
| Daemon | None | dockerd required |
| Root | Not needed | Needs root or group |
| CLI | Docker-compatible | Native |
| Images | Same OCI images | Same OCI images |
| Compose | podman-compose | docker-compose |

## Quick Reference

```bash
podman run -d --name app nginx      # Run container
podman ps                           # List running
podman ps -a                        # List all
podman stop app                     # Stop
podman rm app                       # Remove
podman logs app                     # View logs
podman exec -it app bash            # Shell into container
```

## Use Cases in This Config

| Use Case | Tool | Details |
|----------|------|---------|
| Dev Services | [[dev-services]] | Coder, Jupyter, CloudBeaver |
| Dev Containers | [[dev-containers]] | Full dev environments |
| General containers | Manual | Any container you need |

## Image Management

### Pulling Images

```bash
podman pull nginx                   # From docker.io
podman pull docker.io/library/nginx # Explicit
podman pull quay.io/podman/hello    # From quay.io
podman pull ghcr.io/user/image      # From GitHub
```

### Listing Images

```bash
podman images                       # Local images
podman image ls                     # Same
podman search nginx                 # Search registries
```

### Cleaning Up

```bash
podman rmi nginx                    # Remove image
podman image prune                  # Remove unused
podman system prune                 # Full cleanup
podman system prune -a              # Remove ALL unused
```

## Running Containers

### Basic Run

```bash
# Foreground
podman run nginx

# Detached (background)
podman run -d nginx

# Named container
podman run -d --name myapp nginx

# Interactive
podman run -it alpine sh
```

### Port Forwarding

```bash
# Host:Container
podman run -d -p 8080:80 nginx      # Bind to all interfaces
podman run -d -p 127.0.0.1:8080:80 nginx  # Localhost only (safer)
```

### Volume Mounts

```bash
# Bind mount
podman run -v ~/data:/app/data nginx

# Named volume
podman volume create mydata
podman run -v mydata:/app/data nginx

# SELinux relabel (safe to always use)
podman run -v ~/data:/app/data:Z nginx
```

### Environment Variables

```bash
podman run -e DATABASE_URL=postgres://... myapp
podman run --env-file .env myapp
```

### Resource Limits

```bash
podman run --memory=512m --cpus=1 nginx
```

## Container Management

### Lifecycle

```bash
podman start myapp                  # Start stopped
podman stop myapp                   # Stop running
podman restart myapp                # Restart
podman kill myapp                   # Force stop
podman rm myapp                     # Remove stopped
podman rm -f myapp                  # Force remove (stops first)
```

### Inspection

```bash
podman ps                           # Running containers
podman ps -a                        # All containers
podman logs myapp                   # Container logs
podman logs -f myapp                # Follow logs
podman inspect myapp                # Full details
podman top myapp                    # Running processes
podman stats                        # Resource usage
```

### Exec Into Container

```bash
podman exec myapp ls /app           # Run command
podman exec -it myapp bash          # Interactive shell
podman exec -it myapp sh            # For alpine/minimal
```

## Networking

### Default Network

Containers can reach:
- Each other by container name (on same network)
- Host via `host.containers.internal`
- Internet (via NAT)

### Custom Networks

```bash
podman network create mynet
podman run --network mynet myapp
```

### Host Network

```bash
podman run --network host nginx     # Use host network directly
```

## Storage

### Where Data Lives

```
~/.local/share/containers/storage/
├── overlay-images/       # Downloaded images
├── overlay/              # Container layers
└── overlay-containers/   # Container metadata
```

### Get Container Filesystem

```bash
podman inspect myapp --format '{{.GraphDriver.Data.MergedDir}}'
```

### Named Volumes

```bash
podman volume create mydata
podman volume ls
podman volume inspect mydata
podman volume rm mydata
```

## Rootless Considerations

### User Namespaces

Containers run as your user, mapped to different UIDs inside.

```bash
# Keep host UID mapping (for volume mounts)
podman run --userns=keep-id ...
```

This is important for dev containers where you want files created inside to be owned by your user.

### Networking

Rootless mode uses `slirp4netns` for networking. Some limitations:
- Can't bind to ports < 1024 (use 8080, not 80)
- Slightly slower than root mode

### Lingering

For containers to run after logout:

```bash
loginctl enable-linger $USER
```

## Common Patterns

### Dev Database

```bash
# PostgreSQL for development
podman run -d \
  --name postgres \
  -e POSTGRES_PASSWORD=dev \
  -p 5432:5432 \
  -v pgdata:/var/lib/postgresql/data \
  postgres:15
```

### Web App with Volume

```bash
podman run -d \
  --name webapp \
  -p 3000:3000 \
  -v .:/app:Z \
  -w /app \
  node:20 npm start
```

### One-Off Commands

```bash
# Run command and remove container
podman run --rm alpine cat /etc/os-release
podman run --rm -v .:/app python:3 python /app/script.py
```

## Podman Compose

For multi-container setups:

```bash
# Install
pip install podman-compose

# Use like docker-compose
podman-compose up -d
podman-compose down
podman-compose logs
```

## Troubleshooting

### Container Won't Start

```bash
podman logs myapp                   # Check logs
podman inspect myapp                # Check config
```

### Permission Denied on Volume

Use `:Z` suffix or `--userns=keep-id`:

```bash
podman run -v ~/data:/app:Z myapp
podman run --userns=keep-id -v ~/data:/app myapp
```

### Out of Space

```bash
podman system df                    # Check usage
podman system prune -a              # Clean up
```

### Slow Network

For better performance on rootless:

```bash
podman run --network=pasta ...      # Faster than slirp4netns
```

## Files

| Path | Purpose |
|------|---------|
| `~/.local/share/containers/storage/` | All container data |
| `~/.config/containers/registries.conf` | Registry config |
| `~/.config/containers/storage.conf` | Storage config |

See also: [[dev-containers]], [[dev-services]], [[incus]]
