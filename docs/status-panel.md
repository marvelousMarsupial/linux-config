# Status Panel

Quick system status popup. Press `$mod+i`.

## What it shows

- ASCII clock (large time display)
- Date
- Network (WiFi SSID or ethernet/offline)
- VPN status (wg0 interface)
- Battery level and charging state

## Keybinding

`$mod+i` - opens fuzzel popup with status info

## Script

Located at `~/linux-config/bin/status-panel`

Uses `toilet` for ASCII art clock, checks `/sys/class/power_supply/BAT0` for battery.
