# Attack-Box

CTF/pentesting workstation with GUI (Wireshark, Burp).

## Setup

```bash
forge setup attack-box
forge vm attack-box      # as VM
```

## Inheritance

```
server → desktop → remote-desktop → attack-box
```

Skips: firewall, personal apps (Element, Obsidian, KeePassXC)

## Key Tools

| Tool | Purpose |
|------|---------|
| burpsuite | Web proxy |
| wireshark | Packet capture |
| nmap | Port scanning |
| sqlmap | SQL injection |
| hashcat | Hash cracking |
| hydra | Online brute force |

## Browsers

`firefox-esr`, `chromium` - proxy through Burp at 127.0.0.1:8080

## Wordlists

```
/usr/share/seclists/
/usr/share/wordlists/
```

## Remote Access

```bash
remote-start              # Start headless Sway + VNC
vncviewer 10.0.0.1:5900   # Connect (after WireGuard)
```

## Files

| Path | Purpose |
|------|---------|
| `/opt/burpsuite/burpsuite.jar` | Burp Suite |
| `/usr/local/bin/burpsuite` | Burp wrapper |

See also: [[roles]], [[burp-setup]]
