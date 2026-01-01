#!/bin/bash
# Bootstrap script for linux-config
# Usage: curl -sL <url>/deploy/install.sh | bash -s -- [role]
set -e

ROLE="${1:-server}"
REPO_URL="http://git.rat/August/linux-config.git"
INSTALL_DIR="$HOME/linux-config"

echo "==> Installing $ROLE role"

# Install ansible if not present
if ! command -v ansible-playbook &> /dev/null; then
    echo "==> Installing Ansible..."
    sudo apt-get update
    sudo apt-get install -y ansible git
fi

# Clone or update repo
if [ -d "$INSTALL_DIR" ]; then
    echo "==> Updating repository..."
    cd "$INSTALL_DIR" && git pull
else
    echo "==> Cloning repository..."
    git clone "$REPO_URL" "$INSTALL_DIR"
fi

# Add bin to PATH for this session
export PATH="$INSTALL_DIR/bin:$PATH"

# Run setup via CLI
"$INSTALL_DIR/bin/forge" setup "$ROLE"

echo ""
echo "==> Add to your PATH: export PATH=\"\$HOME/linux-config/bin:\$PATH\""
echo "==> Then use: forge status, forge switch, forge teardown"
echo "==> Log out and back in, or run: exec zsh"
