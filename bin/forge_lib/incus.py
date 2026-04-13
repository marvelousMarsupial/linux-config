"""Incus container/VM management commands."""

import subprocess
import sys
import time
from typing import Optional

from . import DEFAULT_IMAGE, SCRIPT_DIR, Role
from .common import command_exists, run


def generate_instance_name(role: Role, instance_type: str) -> str:
    """Generate instance name as role-type (e.g., server-vm, attack-box-lxc)."""
    return f"{role.value}-{instance_type}"


def incus_instance_exists(name: str) -> bool:
    """Check if an incus instance exists."""
    result = subprocess.run(
        ["incus", "list", "--format=csv", "-c", "n"],
        capture_output=True, text=True
    )
    return name in result.stdout.split()


def incus_wait_ready(name: str, timeout: int = 60) -> bool:
    """Wait for instance to be ready (has network)."""
    print(f"==> Waiting for {name} to be ready...", end="", flush=True)
    for _ in range(timeout):
        result = subprocess.run(
            ["incus", "exec", name, "--", "cat", "/etc/os-release"],
            capture_output=True
        )
        if result.returncode == 0:
            print(" ready!")
            return True
        print(".", end="", flush=True)
        time.sleep(1)
    print(" timeout!")
    return False


def incus_get_ip(name: str) -> Optional[str]:
    """Get the IP address of an incus instance."""
    result = subprocess.run(
        ["incus", "list", name, "--format=csv", "-c", "4"],
        capture_output=True, text=True
    )
    # Parse IP from output like "eth0: 10.x.x.x (family)"
    for line in result.stdout.strip().split("\n"):
        if "eth0" in line or line.strip():
            parts = line.split()
            for part in parts:
                if part.count(".") == 3:  # IPv4
                    return part.rstrip(",")
    return None


def cmd_incus_create(instance_type: str, role: Role, name: Optional[str] = None,
                     image: str = DEFAULT_IMAGE) -> None:
    """Create and provision an Incus container or VM."""
    if not command_exists("incus"):
        print("Error: incus not found. Install with 'forge setup server'.")
        sys.exit(1)

    # Instance type short name for naming
    type_short = "vm" if instance_type == "vm" else "lxc"

    # Generate name if not provided (format: role-type, e.g., server-vm)
    if name is None:
        name = generate_instance_name(role, type_short)

    # Check if already exists
    if incus_instance_exists(name):
        print(f"Error: Instance '{name}' already exists.")
        print(f"Use 'incus delete {name}' to remove it first.")
        sys.exit(1)

    # Create instance
    print(f"==> Creating {instance_type} '{name}' from {image}...")
    cmd = ["incus", "launch", image, name]
    if instance_type == "vm":
        cmd.append("--vm")
    run(cmd)

    # Wait for it to be ready
    if not incus_wait_ready(name):
        print(f"Error: Instance '{name}' failed to start.")
        sys.exit(1)

    # Install prerequisites
    print(f"==> Installing prerequisites in {name}...")
    run(["incus", "exec", name, "--", "apt-get", "update", "-qq"])
    run(["incus", "exec", name, "--", "apt-get", "install", "-y", "-qq",
         "git", "ansible", "sudo"])

    # Push the repo into the instance
    print(f"==> Pushing linux-config to {name}...")
    run(["incus", "file", "push", "-r", str(SCRIPT_DIR), f"{name}/root/"])

    # Run ansible directly with instance_user and instance_hostname overrides
    print(f"==> Running ansible with user={name} hostname={name}...")
    run(["incus", "exec", name, "--", "bash", "-c",
         f"cd /root/linux-config && ansible-playbook -i ansible/inventory.yml "
         f"ansible/playbook.yml -e role={role.value} "
         f"-e instance_user={name} -e instance_hostname={name}"])

    # Get IP and display info
    ip = incus_get_ip(name)
    print()
    print(f"==> Instance '{name}' ready!")
    print(f"    Type:     {instance_type}")
    print(f"    Role:     {role.value}")
    print(f"    User:     {name}")
    print(f"    Hostname: {name}")
    if ip:
        print(f"    IP:       {ip}")
    print()
    print(f"    Shell:  incus exec {name} -- su - {name}")
    print(f"    Delete: incus delete {name} --force")


def cmd_incus_list() -> None:
    """List all incus instances."""
    run(["incus", "list"])


def cmd_incus_delete(name: str, force: bool = False) -> None:
    """Delete an incus instance."""
    if not incus_instance_exists(name):
        print(f"Error: Instance '{name}' does not exist.")
        sys.exit(1)

    cmd = ["incus", "delete", name]
    if force:
        cmd.append("--force")

    print(f"==> Deleting {name}...")
    run(cmd)
    print(f"==> Instance '{name}' deleted.")
