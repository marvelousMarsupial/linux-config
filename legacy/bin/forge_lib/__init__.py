"""forge_lib - Forge library modules."""

from enum import Enum
from pathlib import Path

# Path constants
SCRIPT_DIR = Path(__file__).resolve().parent.parent.parent
ANSIBLE_DIR = SCRIPT_DIR / "ansible"
BIN_DIR = SCRIPT_DIR / "bin"
CACHE_DIR = Path.home() / ".cache" / "forge"
STOW_DIR = SCRIPT_DIR / "stow"

# Default Incus image
DEFAULT_IMAGE = "images:debian/trixie"


class Role(Enum):
    SERVER = "server"
    DEV = "dev"
    DESKTOP = "desktop"
    REMOTE_DESKTOP = "remote-desktop"
    NET_BOX = "net-box"
    ATTACK_BOX = "attack-box"


# Role hierarchy: what roles are included in each role
ROLE_CHAIN = {
    Role.SERVER: [Role.SERVER],
    Role.DEV: [Role.SERVER, Role.DEV],
    Role.DESKTOP: [Role.SERVER, Role.DESKTOP],
    Role.REMOTE_DESKTOP: [Role.SERVER, Role.DESKTOP, Role.REMOTE_DESKTOP],
    Role.NET_BOX: [Role.SERVER, Role.NET_BOX],
    Role.ATTACK_BOX: [Role.SERVER, Role.DESKTOP, Role.REMOTE_DESKTOP, Role.ATTACK_BOX],
}

# What to teardown when switching FROM a role
TEARDOWN_CHAIN = {
    Role.SERVER: [Role.SERVER],
    Role.DEV: [Role.DEV],
    Role.DESKTOP: [Role.DESKTOP],
    Role.REMOTE_DESKTOP: [Role.REMOTE_DESKTOP, Role.DESKTOP],
    Role.NET_BOX: [Role.NET_BOX],
    Role.ATTACK_BOX: [Role.ATTACK_BOX, Role.REMOTE_DESKTOP, Role.DESKTOP],
}

# Dev container constants
DEV_CONTAINER_PREFIX = "dev-"
DEV_RUNTIMES = ["python", "c", "ts", "nuxt", "bun", "java", "base"]

# Packages for each runtime
RUNTIME_PACKAGES = {
    "base": [],
    "python": ["python3", "python3-pip", "python3-venv", "ipython3"],
    "c": ["gcc", "g++", "gdb", "make", "cmake", "clang"],
    "ts": [],  # Node installed via nodesource
    "nuxt": [],  # Node installed via nodesource
    "bun": [],  # Bun installed via curl script
    "java": ["openjdk-17-jdk", "maven", "gradle"],
}
