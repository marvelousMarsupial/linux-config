# Incus

System container and VM manager. LXD fork, managed by Linux Containers.

## vs Podman

| Feature | Incus | Podman |
|---------|-------|--------|
| Type | System containers, VMs | Application containers |
| Init | Full init system (systemd) | Single process |
| Use case | Full OS isolation | Running apps |
| Storage | ZFS, btrfs, LVM | Overlay |
| Networking | Full bridge, VLAN | NAT, slirp |

Use Podman for apps. Use Incus for VMs or full OS environments.

## Quick Reference

```bash
incus launch images:debian/12 mybox     # Create container
incus launch images:debian/12 myvm --vm # Create VM
incus list                              # List instances
incus exec mybox -- bash                # Shell access
incus stop mybox                        # Stop
incus delete mybox                      # Remove
```

## Setup

### First Time

```bash
incus admin init
```

Accept defaults for simple setup, or customize:
- Storage pool (dir, zfs, btrfs)
- Network bridge (incusbr0)
- Clustering (no for single machine)

### Forge Integration

[[forge]] can create provisioned instances:

```bash
forge lxc server                  # Container with server role
forge vm desktop                  # VM with desktop role
forge instances                   # List forge-created instances
forge delete mybox                # Remove instance
```

## Images

### Listing Available

```bash
incus image list images:          # All available
incus image list images: debian   # Filter by name
incus image list images: alpine
```

### Common Images

| Image | Description |
|-------|-------------|
| images:debian/12 | Debian Bookworm |
| images:debian/trixie | Debian Testing |
| images:ubuntu/22.04 | Ubuntu LTS |
| images:ubuntu/24.04 | Ubuntu LTS |
| images:alpine/3.19 | Alpine Linux |
| images:archlinux | Arch Linux |

### Local Images

```bash
incus image list                  # Local images
incus image delete <fingerprint>  # Remove local
```

## Containers

### Creating

```bash
# Basic
incus launch images:debian/12 mybox

# With limits
incus launch images:debian/12 mybox -c limits.cpu=2 -c limits.memory=2GB

# From specific architecture
incus launch images:debian/12/arm64 mybox
```

### Management

```bash
incus start mybox
incus stop mybox
incus restart mybox
incus delete mybox
incus delete mybox --force        # Stop and delete
```

### Shell Access

```bash
incus exec mybox -- bash          # Root shell
incus exec mybox -- su - august   # As user
incus exec mybox -- /bin/sh       # For alpine
```

### Configuration

```bash
incus config show mybox           # View config
incus config set mybox limits.cpu=4
incus config set mybox limits.memory=4GB
incus config unset mybox limits.cpu
```

## VMs

### Creating

```bash
# Basic VM
incus launch images:debian/12 myvm --vm

# With resources
incus launch images:debian/12 myvm --vm \
  -c limits.cpu=4 \
  -c limits.memory=8GB \
  -d root,size=50GB
```

### VM-Specific

```bash
incus console myvm                # VM console
incus console myvm --type=vga     # Graphical console (if GUI)
```

### When to Use VMs

- Need different kernel
- Running Docker inside (nested containers)
- Security isolation
- Testing kernel modules

## File Transfer

### Push/Pull

```bash
incus file push local.txt mybox/root/
incus file push -r mydir/ mybox/root/

incus file pull mybox/root/file.txt .
incus file pull -r mybox/root/mydir/ .
```

### Edit In Place

```bash
incus file edit mybox/etc/hosts
```

## Networking

### Default Bridge

Containers get IP on incusbr0 bridge, NAT to internet.

```bash
incus list                        # Shows IP addresses
```

### Port Forwarding

```bash
# Forward host:8080 to container:80
incus config device add mybox http proxy \
  listen=tcp:0.0.0.0:8080 \
  connect=tcp:127.0.0.1:80
```

### Static IP

```bash
incus config device set mybox eth0 ipv4.address=10.0.0.100
```

## Snapshots

### Creating

```bash
incus snapshot create mybox snap1
incus snapshot create mybox snap-clean --stateful  # With memory
```

### Listing

```bash
incus snapshot list mybox
```

### Restoring

```bash
incus snapshot restore mybox snap1
```

### Deleting

```bash
incus snapshot delete mybox snap1
```

## Profiles

Profiles are reusable configuration:

```bash
incus profile list
incus profile show default
incus profile copy default myprofile
incus profile edit myprofile
incus launch images:debian/12 mybox --profile myprofile
```

### Custom Profile Example

```yaml
config:
  limits.cpu: "2"
  limits.memory: 2GB
devices:
  root:
    path: /
    pool: default
    size: 20GB
    type: disk
```

## Storage

### Pools

```bash
incus storage list
incus storage info default
```

### Custom Disk Size

```bash
incus launch images:debian/12 mybox -d root,size=50GB
```

### Adding Disk

```bash
incus config device add mybox data disk \
  source=/path/on/host \
  path=/mnt/data
```

## Cloud-Init

For automated setup:

```bash
incus launch images:debian/12 mybox --config=user.user-data="$(cat cloud-init.yaml)"
```

Example cloud-init.yaml:
```yaml
#cloud-config
packages:
  - vim
  - git
runcmd:
  - echo "Hello from cloud-init"
```

## Troubleshooting

### Container Won't Start

```bash
incus info mybox --show-log
```

### Network Not Working

```bash
incus exec mybox -- ping 8.8.8.8  # Check internet
incus exec mybox -- cat /etc/resolv.conf  # Check DNS
```

### Permission Issues

Check user mapping:
```bash
incus config show mybox | grep -A5 raw.idmap
```

### Storage Full

```bash
incus storage info default
incus image list | grep -v CACHED | wc -l
```

## Files

| Path | Purpose |
|------|---------|
| /var/lib/incus/ | Incus data (as root) |
| ~/.config/incus/ | Client config |

See also: [[forge]], [[dev-containers]], [[podman]]
