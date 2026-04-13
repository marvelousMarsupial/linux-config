"""Ansible role management commands."""

import subprocess
import sys
from typing import Optional

from . import ANSIBLE_DIR, ROLE_CHAIN, TEARDOWN_CHAIN, Role
from .common import (
    command_exists,
    ensure_ansible,
    get_current_role,
    run,
    set_current_role,
    ansible_collections_installed,
)


def cmd_setup(role: Role) -> None:
    """Install a role."""
    print(f"==> Setting up {role.value}...")

    ensure_ansible()

    # Install collections if needed
    if not ansible_collections_installed():
        print("==> Installing Ansible collections...")
        run(["ansible-galaxy", "collection", "install", "-r",
             str(ANSIBLE_DIR / "requirements.yml")])

    # Run playbook
    run([
        "ansible-playbook",
        "-i", str(ANSIBLE_DIR / "inventory.yml"),
        str(ANSIBLE_DIR / "playbook.yml"),
        "-e", f"role={role.value}",
        "-K",
    ])

    set_current_role(role)
    print(f"==> Done! Role '{role.value}' is now active.")


def cmd_teardown(role: Optional[Role] = None, full: bool = False) -> None:
    """Remove a role."""
    if role is None:
        role = get_current_role()

    if role is None and not full:
        print("No role currently installed. Use 'forge teardown server --full' to force.")
        return

    # Default to server if doing full teardown with no role
    if role is None:
        role = Role.SERVER

    ensure_ansible()

    # Full teardown: tear down entire role chain (e.g., desktop -> server)
    if full:
        print(f"==> Full teardown of {role.value} (entire chain)...")
        # Reverse the chain so we tear down leaf roles first
        roles_to_teardown = list(reversed(ROLE_CHAIN[role]))
        for r in roles_to_teardown:
            print(f"==> Tearing down {r.value}...")
            cmd = [
                "ansible-playbook",
                "-i", str(ANSIBLE_DIR / "inventory.yml"),
                str(ANSIBLE_DIR / "teardown.yml"),
                "-e", f"role={r.value}",
                "-e", "full_teardown=true",
                "-K",
            ]
            run(cmd, check=False)  # Don't stop on errors
    else:
        print(f"==> Tearing down {role.value}...")
        cmd = [
            "ansible-playbook",
            "-i", str(ANSIBLE_DIR / "inventory.yml"),
            str(ANSIBLE_DIR / "teardown.yml"),
            "-e", f"role={role.value}",
            "-K",
        ]
        run(cmd)

    set_current_role(None)
    print("==> Teardown complete.")


def cmd_switch(new_role: Role) -> None:
    """Switch from current role to a new role."""
    current = get_current_role()

    if current is None:
        cmd_setup(new_role)
        return

    if current == new_role:
        print(f"Already on {new_role.value}. Running setup to refresh...")
        cmd_setup(new_role)
        return

    print(f"==> Switching from {current.value} to {new_role.value}...")

    # Get what the new role includes
    new_bases = set(ROLE_CHAIN[new_role])

    # Teardown current role chain, skipping roles included in new role
    ensure_ansible()
    for role in TEARDOWN_CHAIN[current]:
        if role not in new_bases:
            print(f"==> Tearing down {role.value}...")
            run([
                "ansible-playbook",
                "-i", str(ANSIBLE_DIR / "inventory.yml"),
                str(ANSIBLE_DIR / "teardown.yml"),
                "-e", f"role={role.value}",
                "-K",
            ])

    cmd_setup(new_role)


def cmd_status() -> None:
    """Show current role and theme."""
    role = get_current_role()
    print(f"Role: {role.value if role else 'none'}")

    if command_exists("theme"):
        print("Theme: ", end="", flush=True)
        result = subprocess.run(["theme", "current"], capture_output=True, text=True)
        if result.returncode == 0:
            print(result.stdout.split("\n")[0])
        else:
            print("not set")


def cmd_cleanup() -> None:
    """Remove ansible after setup."""
    print("==> Removing build dependencies...")
    run(["sudo", "apt-get", "remove", "--purge", "-y", "ansible"])
    run(["sudo", "apt-get", "autoremove", "-y"])
    print("==> Cleanup complete. Dotfiles still installed, ansible removed.")


def cmd_theme(args: list[str]) -> None:
    """Delegate to theme command."""
    if command_exists("theme"):
        run(["theme"] + args)
    else:
        print("Theme command not found. Run 'forge setup' first.")
        sys.exit(1)
