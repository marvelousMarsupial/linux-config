# Ergent befor deploy

[Documentation index](README.md)

## Finding 3: full setup omitted the incoming firewall

The security review on 2026-10-08 found that full setup could finish without
installing or enabling the repository's incoming default-deny firewall.

Fixed in `scripts/setup`: after installing packages, setup validates the rules,
backs up existing firewall configuration under `/var/backups/linux-config/`,
installs both the policy and the nftables service override, and enables and
reloads nftables. It checks service enablement, active state and the presence of
`inet linux_config`; a failure stops setup. Reloading replaces only that table,
preserving other applications' tables. Dry runs print the plan without changes.

Before deploying, review required incoming access and VM bridge exceptions in
[the firewall guide](firewall.md). The supplied policy permits libvirt DNS/DHCP
on `virbr*`, but has no SSH or other incoming service exception. Use local-console
access or a tested recovery path when applying it remotely.

Remaining deployment verification: run setup on the target, test required access
and guest connectivity, then verify the policy survives reboot. Offline tests
cover command sequencing, backups, dry-run behavior and failure propagation;
they do not prove live packet filtering. This fix has not been applied to the
current host during this task.

The other review findings were accepted by the user and are outside this change.
