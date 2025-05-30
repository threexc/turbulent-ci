#!/bin/bash

# Build the project
echo "🔨 Building Turbulent CI..."
cargo build --release

# Create config directory
CONFIG_DIR="$HOME/.config/turbulent-ci"
mkdir -p "$CONFIG_DIR"

# Copy binary to /usr/local/bin (requires sudo)
echo "📦 Installing binary..."
sudo cp target/release/turbulent-ci /usr/local/bin/
sudo chmod +x /usr/local/bin/turbulent-ci

# Create systemd service file
echo "⚙️  Creating systemd service..."
sudo tee /etc/systemd/system/turbulent-ci.service > /dev/null <<EOF
[Unit]
Description=Turbulent CI Multi-Repository Daemon
After=network.target

[Service]
Type=simple
User=$USER
Group=$(id -gn)
WorkingDirectory=$HOME
ExecStart=/usr/local/bin/turbulent-ci start
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd
sudo systemctl daemon-reload

echo "✅ Installation complete!"
echo ""
echo "Usage:"
echo "  turbulent-ci add ./path/to/repo --name 'My Project'"
echo "  turbulent-ci list"
echo "  turbulent-ci start"
echo ""
echo "To run as a service:"
echo "  sudo systemctl enable turbulent-ci"
echo "  sudo systemctl start turbulent-ci"
echo "  sudo systemctl status turbulent-ci"
