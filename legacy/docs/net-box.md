# Net-Box

CLI network monitoring workstation. Blue team.

## Setup

```bash
forge setup net-box
```

## Traffic Capture

### tshark (CLI Wireshark)

```bash
# Capture on interface
sudo tshark -i eth0

# Capture with filter
sudo tshark -i eth0 -f "port 80"

# Read pcap file
tshark -r capture.pcap

# Show HTTP requests
tshark -r capture.pcap -Y "http.request"

# Extract fields
tshark -r capture.pcap -T fields -e ip.src -e ip.dst -e tcp.port

# Live capture to file
sudo tshark -i eth0 -w capture.pcap
```

### tcpdump

```bash
# Basic capture
sudo tcpdump -i eth0

# Capture to file
sudo tcpdump -i eth0 -w capture.pcap

# Read pcap
tcpdump -r capture.pcap

# Filter by host
sudo tcpdump -i eth0 host 192.168.1.1

# Filter by port
sudo tcpdump -i eth0 port 443

# Show packet contents
sudo tcpdump -i eth0 -X

# Don't resolve names (faster)
sudo tcpdump -i eth0 -n
```

### termshark (TUI)

```bash
# Live capture
sudo termshark -i eth0

# Read pcap
termshark -r capture.pcap
```

### tcpflow (Reassemble streams)

```bash
# Extract TCP streams from pcap
tcpflow -r capture.pcap -o output_dir/

# Live capture streams
sudo tcpflow -i eth0 -o output_dir/
```

### ngrep (Grep for packets)

```bash
# Search for string in packets
sudo ngrep -d eth0 "password"

# HTTP traffic
sudo ngrep -d eth0 -W byline port 80

# Case insensitive
sudo ngrep -i -d eth0 "login"
```

## Bandwidth Monitoring

### iftop (Per-connection)

```bash
# Monitor interface
sudo iftop -i eth0

# Don't resolve hostnames
sudo iftop -i eth0 -n

# Show ports
sudo iftop -i eth0 -P
```

Keys: `t` toggle display, `s`/`d` sort by src/dst, `q` quit

### nethogs (Per-process)

```bash
# Monitor all interfaces
sudo nethogs

# Specific interface
sudo nethogs eth0

# Refresh rate
sudo nethogs -d 2
```

### vnstat (Historical stats)

```bash
# Show summary
vnstat

# Live monitor
vnstat -l

# Daily stats
vnstat -d

# Monthly
vnstat -m

# Top 10 days
vnstat -t
```

### bmon (TUI bandwidth)

```bash
bmon
```

Keys: `d` detailed, `g` graph, `q` quit

### iperf3 (Speed test)

```bash
# Server mode
iperf3 -s

# Client test
iperf3 -c server_ip

# UDP test
iperf3 -c server_ip -u

# Reverse (server sends)
iperf3 -c server_ip -R
```

## Network Discovery

### nmap

```bash
# Ping scan (find hosts)
nmap -sn 192.168.1.0/24

# Port scan
nmap 192.168.1.1

# Service detection
nmap -sV 192.168.1.1

# OS detection
sudo nmap -O 192.168.1.1

# Full scan
sudo nmap -A 192.168.1.1

# Scan specific ports
nmap -p 22,80,443 192.168.1.1

# Fast scan (top 100 ports)
nmap -F 192.168.1.0/24

# Save to file
nmap -oN scan.txt 192.168.1.1
```

### arp-scan

```bash
# Scan local network
sudo arp-scan -l

# Scan range
sudo arp-scan 192.168.1.0/24

# Specific interface
sudo arp-scan -I eth0 -l
```

### netdiscover

```bash
# Passive mode (listen only)
sudo netdiscover -p

# Active scan
sudo netdiscover -r 192.168.1.0/24
```

## IDS (Suricata)

### Setup

```bash
# Update rules
sudo suricata-update

# Test config
sudo suricata -T -c /etc/suricata/suricata.yaml

# Run on interface
sudo suricata -c /etc/suricata/suricata.yaml -i eth0
```

### Logs

```bash
# Fast log (alerts)
tail -f /var/log/suricata/fast.log

# JSON log
tail -f /var/log/suricata/eve.json | jq

# Parse alerts
cat /var/log/suricata/eve.json | jq 'select(.event_type=="alert")'
```

### Rules

```bash
# Rule location
/var/lib/suricata/rules/

# Custom rules
/etc/suricata/rules/local.rules

# Example rule
alert tcp any any -> any 22 (msg:"SSH connection"; sid:1000001; rev:1;)
```

## ARP Monitoring

### arpwatch

```bash
# Start daemon
sudo systemctl start arpwatch

# View log
tail -f /var/log/syslog | grep arpwatch

# Check for new devices, IP changes, flip-flops
```

### arping

```bash
# Ping by IP
sudo arping 192.168.1.1

# Ping by MAC
sudo arping -T 192.168.1.1 00:11:22:33:44:55

# Detect duplicates
sudo arping -D 192.168.1.1
```

## DNS & Routing

### dig

```bash
# Basic lookup
dig example.com

# Short answer
dig +short example.com

# Specific record
dig example.com MX
dig example.com TXT

# Reverse lookup
dig -x 8.8.8.8

# Use specific server
dig @8.8.8.8 example.com

# Trace
dig +trace example.com
```

### mtr (Traceroute + ping)

```bash
# Basic
mtr google.com

# Report mode
mtr -r google.com

# No DNS
mtr -n google.com
```

## Utilities

### netcat

```bash
# Listen on port
nc -l 4444

# Connect to port
nc host 4444

# Port scan
nc -zv host 20-30

# Transfer file
nc -l 4444 > file.txt        # receiver
nc host 4444 < file.txt      # sender

# Chat
nc -l 4444                   # server
nc host 4444                 # client
```

### socat

```bash
# TCP listener
socat TCP-LISTEN:4444 -

# Connect
socat - TCP:host:4444

# Port forward
socat TCP-LISTEN:8080,fork TCP:remote:80

# TLS listener
socat OPENSSL-LISTEN:443,cert=cert.pem -
```

### ss (Socket stats)

```bash
# All connections
ss -a

# Listening ports
ss -l

# TCP only
ss -t

# With process
ss -p

# Numeric (no resolve)
ss -n

# Combined (listening TCP with process)
ss -tlnp
```

## Quick Reference

| Task | Command |
|------|---------|
| Capture packets | `sudo tcpdump -i eth0 -w out.pcap` |
| Analyze pcap | `tshark -r out.pcap` |
| Live bandwidth | `sudo iftop -i eth0` |
| Per-process BW | `sudo nethogs` |
| Find hosts | `sudo arp-scan -l` |
| Port scan | `nmap -F target` |
| Check alerts | `tail -f /var/log/suricata/fast.log` |
| DNS lookup | `dig +short example.com` |
| Traceroute | `mtr target` |
| Listen port | `nc -l 4444` |
| Connections | `ss -tlnp` |

See also: [[roles]], [[attack-box]]
