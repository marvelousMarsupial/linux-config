"""Common utility functions for forge."""

import subprocess
import sys
from typing import Optional

from . import CACHE_DIR, Role


def run(cmd: list[str], check: bool = True) -> subprocess.CompletedProcess:
    """Run a command and return the result."""
    return subprocess.run(cmd, check=check)


def run_output(cmd: list[str]) -> str:
    """Run a command and return stdout."""
    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.stdout.strip()


def run_quiet(cmd: list[str]) -> bool:
    """Run a command silently, return True if successful."""
    return subprocess.run(cmd, capture_output=True).returncode == 0


def command_exists(cmd: str) -> bool:
    """Check if a command exists in PATH."""
    return subprocess.run(["which", cmd], capture_output=True).returncode == 0


def ensure_ansible() -> None:
    """Install ansible if not present."""
    if not command_exists("ansible-playbook"):
        print("==> Installing Ansible...")
        run(["sudo", "apt-get", "update", "-qq"])
        run(["sudo", "apt-get", "install", "-y", "-qq", "ansible"])


def get_current_role() -> Optional[Role]:
    """Get the currently installed role."""
    role_file = CACHE_DIR / "role"
    if role_file.exists():
        role_str = role_file.read_text().strip()
        if role_str and role_str != "none":
            try:
                return Role(role_str)
            except ValueError:
                return None
    return None


def set_current_role(role: Optional[Role]) -> None:
    """Set the current role in cache."""
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    role_file = CACHE_DIR / "role"
    role_file.write_text(role.value if role else "none")


def parse_role(role_str: str) -> Role:
    """Parse a role string into a Role enum."""
    try:
        return Role(role_str)
    except ValueError:
        valid = ", ".join(r.value for r in Role)
        print(f"Error: Invalid role '{role_str}'")
        print(f"Valid roles: {valid}")
        sys.exit(1)


def ansible_collections_installed() -> bool:
    """Check if required ansible collections are installed."""
    result = subprocess.run(
        ["ansible-galaxy", "collection", "list"],
        capture_output=True, text=True
    )
    return "community.general" in result.stdout


def cmd_cleanup_home(dry_run: bool = False) -> None:
    """Remove unnecessary files from home directory."""
    from pathlib import Path
    import shutil

    home = Path.home()

    # Files/dirs to remove
    cleanup_targets = [
        # Temp/cache that rebuilds
        ".ansible",
        ".npm",
        ".wget-hsts",
        # Unused with zsh
        ".bash_logout",
        ".profile",
        # Ubuntu markers
        ".sudo_as_admin_successful",
        # Old/unused
        ".lesshst",
        ".python_history",
    ]

    removed = []
    for target in cleanup_targets:
        path = home / target
        if path.exists():
            if dry_run:
                print(f"  Would remove: {path}")
            else:
                if path.is_dir():
                    shutil.rmtree(path)
                else:
                    path.unlink()
                removed.append(target)

    if dry_run:
        print("\nRun without --dry-run to remove these files.")
    elif removed:
        print(f"Removed: {', '.join(removed)}")
    else:
        print("Nothing to clean up.")
