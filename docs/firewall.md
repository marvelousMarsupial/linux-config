# Host firewall

[Documentation index](README.md)

The workstation uses nftables with a dedicated `inet linux_config` table for
IPv4 and IPv6. Incoming traffic is dropped unless it is loopback, a reply to an
established/related connection, ICMP/ICMPv6, a DHCP client reply, or a
DNS/DHCP request arriving on a libvirt `virbr*` bridge. Invalid tracked
packets are dropped.

Outgoing traffic is unrestricted by this table. Guest forwarding and NAT remain
under libvirt's control. There is no SSH, web-server, LLMNR or
multicast-discovery exception. Sharing a service deliberately requires adding a
narrowly scoped rule. This is host protection, not an outbound application
firewall or a VM isolation policy. OpenSnitch covers outbound prompts; see
[system health](sysreport.md#opensnitch).

## What setup does

Full `scripts/setup` installs and activates this policy right after package
installation, before anything is downloaded. It validates the source with
`nft --check`, backs up both existing files under
`/var/backups/linux-config/<timestamp>/firewall/`, installs the policy and the
service override, enables and starts nftables, then reloads it. It checks that
the service is enabled and active and that the project table exists; a failure
stops setup. Setup is not transactional: backups remain, but a later failed
step does not undo an already applied firewall. The config-only installer
leaves the firewall unchanged.

Review required incoming access before setup. There is no default SSH
exception, so a remotely managed machine needs local-console access or a
tested recovery path before the policy is applied.

The config atomically flushes and rebuilds only its own table on reload. The
service override replaces Debian's global `ExecStop` flush with deletion of only
this table, preserving libvirt and other applications' rules on stop or restart.
Stopping nftables disables this host protection.

## Sharing a service on a private network

The policy has a marked spot before the final drop for narrowly scoped
exceptions. Add one rule per service, matching the trusted source subnet and
the TCP or UDP port, and never trust a whole interface. For example, to allow
SSH from one private subnet, add a line with your own subnet in place of the
documentation range:

```text
ip saddr 192.0.2.0/24 tcp dport 22 counter accept
```

Such a rule does not forward a router port or expose IPv6, and the service
must also listen on the host's private address.

Keep machine-specific subnets out of the public repository: edit the installed
`/etc/nftables.conf`, or keep a private copy of the policy, and reapply it
after a setup run replaces the file. To apply an updated policy to a machine
that already has the firewall, run from the repository root:

```sh
sudo nft --check --file system/nftables.conf
sudo cp -a /etc/nftables.conf /etc/nftables.conf.before
sudo install -m 0644 system/nftables.conf /etc/nftables.conf
sudo systemctl reload nftables
sudo nft list chain inet linux_config input
```

Then test the connection from a device on the allowed subnet. Removing the
rule and reloading the policy closes the access again.

## Inspect and change

```sh
sudo nft list table inet linux_config
systemctl status nftables
sudo nft --check --file /etc/nftables.conf
sudo systemctl reload nftables
```

Project sources are `system/nftables.conf` and
`system/systemd/nftables.service.d/10-linux-config.conf`. Both must be
installed together. For manual maintenance on a machine whose existing
firewall you have inspected and backed up:

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

To allow SSH from a trusted network, add a rule before the final drop matching
the actual interface, source subnet and TCP port. Validate and reload. Do not
add a broad trust rule for the whole physical interface.

Emergency recovery from a local console:

```sh
sudo systemctl stop nftables
```

This removes only the project's table. Correct the configuration and run
`sudo systemctl start nftables` to restore protection. Never use a global
`nft flush ruleset` on this libvirt workstation.

## Verification record

On the development VM (2026-10-07) the policy was installed, enabled and
tested from a disposable network namespace: a new TCP connection to a listening
port was blocked, loopback stayed available, a host-initiated connection
received its reply, and DNS plus HTTPS worked. Libvirt's tables remained
present after activation and reload. IPv6 rules loaded but were not probed
separately, no running-guest connectivity test was done, and no reboot was
performed. The pre-install configuration is in
`/var/backups/linux-config-firewall/` (root only).

Still pending on the target machine: a run of setup, required access and
guest connectivity, any local service exception, and the policy surviving a
reboot. See [TODO](TODO.md).

Reference: [Netfilter's workstation firewall example](https://wiki.nftables.org/wiki-nftables/index.php/Simple_ruleset_for_a_workstation).
