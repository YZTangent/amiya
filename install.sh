#!/bin/bash
# Amiya Installation Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Amiya Installation Script ===${NC}"
echo

# Check for required dependencies
echo "Checking dependencies..."

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo is not installed.${NC}"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi
echo -e "${GREEN}✓${NC} Cargo found"

# Check for GTK4 development files
if ! pkg-config --exists gtk4 2>/dev/null; then
    echo -e "${YELLOW}Warning: GTK4 development files not found.${NC}"
    echo "Please install GTK4 development files:"
    echo "  Arch: sudo pacman -S gtk4"
    echo "  Ubuntu/Debian: sudo apt install libgtk-4-dev"
    echo "  Fedora: sudo dnf install gtk4-devel"
    exit 1
fi
echo -e "${GREEN}✓${NC} GTK4 found"

# Check for other required libraries
for lib in "gdk-pixbuf-2.0" "pango" "cairo"; do
    if ! pkg-config --exists "$lib" 2>/dev/null; then
        echo -e "${YELLOW}Warning: $lib not found${NC}"
    fi
done

echo
echo "Building Amiya..."
cargo build --release

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✓${NC} Build successful"

echo
echo "Installing binaries..."

# Create installation directories
mkdir -p ~/.local/bin
mkdir -p ~/.config/systemd/user
mkdir -p ~/.config/amiya

# Install binaries
cp target/release/amiya ~/.local/bin/
cp target/release/amiya-ctl ~/.local/bin/
chmod +x ~/.local/bin/amiya
chmod +x ~/.local/bin/amiya-ctl
echo -e "${GREEN}✓${NC} Installed binaries to ~/.local/bin/"

# Install systemd service
cp amiya.service ~/.config/systemd/user/
echo -e "${GREEN}✓${NC} Installed systemd service"

# Install default configuration if it doesn't exist
if [ ! -f ~/.config/amiya/config.toml ]; then
    cp amiya.toml ~/.config/amiya/config.toml
    echo -e "${GREEN}✓${NC} Installed default configuration"
else
    echo -e "${YELLOW}!${NC} Configuration file already exists, skipping"
fi

echo
echo -e "${GREEN}=== Installation Complete ===${NC}"
echo
echo "Next steps:"
echo "  1. Make sure ~/.local/bin is in your PATH:"
echo "     export PATH=\"\$HOME/.local/bin:\$PATH\""
echo
echo "  2. Start Amiya manually:"
echo "     amiya"
echo
echo "  3. Or enable auto-start with systemd:"
echo "     systemctl --user enable --now amiya.service"
echo
echo "  4. View logs:"
echo "     journalctl --user -u amiya -f"
echo
echo "  5. Configure hotkeys in your window manager:"
echo "     - For niri: see docs/niri-config-example.kdl"
echo "     - For swhkd: see docs/swhkd-config-example"
echo
echo "  6. Test the CLI:"
echo "     amiya-ctl status"
echo "     amiya-ctl popup toggle bluetooth"
echo
echo "For more information, see the README.md file."
