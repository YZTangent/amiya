#!/bin/bash
# Amiya Uninstallation Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== Amiya Uninstallation Script ===${NC}"
echo

# Stop and disable systemd service if running
if systemctl --user is-active amiya.service &>/dev/null; then
    echo "Stopping Amiya service..."
    systemctl --user stop amiya.service
    echo -e "${GREEN}✓${NC} Service stopped"
fi

if systemctl --user is-enabled amiya.service &>/dev/null; then
    echo "Disabling Amiya service..."
    systemctl --user disable amiya.service
    echo -e "${GREEN}✓${NC} Service disabled"
fi

# Remove binaries
if [ -f ~/.local/bin/amiya ]; then
    rm ~/.local/bin/amiya
    echo -e "${GREEN}✓${NC} Removed amiya binary"
fi

if [ -f ~/.local/bin/amiya-ctl ]; then
    rm ~/.local/bin/amiya-ctl
    echo -e "${GREEN}✓${NC} Removed amiya-ctl binary"
fi

# Remove systemd service
if [ -f ~/.config/systemd/user/amiya.service ]; then
    rm ~/.config/systemd/user/amiya.service
    systemctl --user daemon-reload
    echo -e "${GREEN}✓${NC} Removed systemd service"
fi

# Ask about configuration
echo
read -p "Remove configuration files? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d ~/.config/amiya ]; then
        rm -rf ~/.config/amiya
        echo -e "${GREEN}✓${NC} Removed configuration directory"
    fi
else
    echo -e "${YELLOW}!${NC} Kept configuration files in ~/.config/amiya"
fi

# Clean up IPC socket
if [ -d "${XDG_RUNTIME_DIR:-/tmp}/amiya" ]; then
    rm -rf "${XDG_RUNTIME_DIR:-/tmp}/amiya"
    echo -e "${GREEN}✓${NC} Cleaned up IPC socket"
fi

echo
echo -e "${GREEN}=== Uninstallation Complete ===${NC}"
echo
echo "Amiya has been removed from your system."
