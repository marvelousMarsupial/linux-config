# Host firewall

[Documentation index](README.md)

The workstation uses nftables with a dedicated `inet linux_config` table for
IPv4 and IPv6. Incoming traffic is dropped unless it is loopback, a reply to
an established/related connection, ICMP/ICMPv6, DHCP client replies, or DNS/DHCP
requests arriving on a libvirt `virbr*` bridge. Invalid tracked packets are dropped.

Outgoing traffic is unrestricted by this table. Guest forwarding and NAT remain
under libvirt's control. There is no SSH, web-server, LLMNR or multicast-discovery
exception. Deliberately sharing a service requires adding a narrowly scoped rule.
ICMP remains available for diagnostics, IPv6 discovery and path MTU handling.
This is host protection, not an outbound application firewall or a VM isolation policy.

The config atomically flushes and rebuilds only its own table on reload. The
service override replaces Debian's global `ExecStop` flush with deletion of only
this table, preserving libvirt and other applications' rules on stop/restart.
Stopping nftables disables this host protection.

## Installed machine — 2026-10-07

Installed and enabled on the current VM. The Incus guest agent provides the
management path; no SSH listener was present. Libvirt's default network was
active and its IPv4/IPv6 tables remained present after firewall activation/reload.

Live tests using a disposable network namespace verified that a new TCP
connection to the host's listening LLMNR port was blocked, loopback access
remained available, and a host-initiated TCP connection received its reply.
DNS resolution and HTTPS to Debian worked after activation. nftables config
validation passed. IPv6 rules loaded, but no separate IPv6 traffic probe or
running guest connectivity test was performed. Boot persistence is configured;
a reboot was not performed.

The pre-install configuration and ruleset are in
`/var/backups/linux-config-firewall/` (root only). The old config was an empty
Forge placeholder; pre-existing tables belonged to libvirt.

## Machine-local outbound trial

OpenSnitch packages, UI theme defaults, launcher and user service are now included
in the repository. Setup provisions the daemon/UI socket through a systemd
override; other daemon preferences and allow rules remain local. It can block
outgoing traffic that this host firewall permits. The checks above predate that
trial. See [system health](sysreport.md#opensnitch) for database and socket setup;
the user confirmed connection collection on 2026-10-08.

## Inspect and change

```sh
sudo nft list table inet linux_config
systemctl status nftables
sudo nft --check --file /etc/nftables.conf
sudo systemctl reload nftables
```

Project sources are `system/nftables.conf` and
`system/systemd/nftables.service.d/10-linux-config.conf`. Both must be installed
when configuring another machine. Inspect its existing firewall and management
path first. The user-file installer does not apply system configuration.

After inspecting a target machine and backing up its existing configuration,
install both files from the repository root:

```sh
sudo nft --check --file system/nftables.conf
sudo install -m 0644 system/nftables.conf /etc/nftables.conf
sudo install -D -m 0644 system/systemd/nftables.service.d/10-linux-config.conf /etc/systemd/system/nftables.service.d/10-linux-config.conf
sudo systemctl daemon-reload
sudo systemctl enable nftables
# start for a new installation; reload if already active
sudo systemctl start nftables
sudo systemctl reload nftables
```

On a remotely managed machine, arrange local-console access or a timed rollback
before loading rules that may block its management connection.

For example, to allow SSH from a trusted network, add a rule before the final
drop matching the actual interface, source subnet and TCP port. Validate and
reload. Do not add a broad trust rule for the whole physical interface.

Emergency recovery from a local console:

```sh
sudo systemctl stop nftables
```

This removes only the project's table. Correct the configuration and run
`sudo systemctl start nftables` to restore protection. Never use a global
`nft flush ruleset` on this libvirt workstation.

Reference: [Netfilter's workstation firewall example](https://wiki.nftables.org/wiki-nftables/index.php/Simple_ruleset_for_a_workstation).
