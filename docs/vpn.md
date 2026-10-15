# VPN

[Documentation index](README.md)

Super+Shift+W opens the VPN menu. It lists every WireGuard profile known to
NetworkManager and, when the client is installed, Tailscale. A filled circle
marks an active tunnel; choosing a row toggles it. "Disconnect all" appears
while anything is up.

## Fail off

- Nothing connects at boot or login. Imported profiles get `autoconnect no`,
  and `vpn-off.service` runs `vpn-menu off` when the Sway session starts, which
  also stops a Tailscale state restored by its daemon.
- A tunnel that fails to come up is brought down again before the error is
  reported, so no routes or interface are left half configured.
- Disconnect failures are reported as critical notifications instead of being
  ignored.

## WireGuard

Put a provider or server `.conf` file in `~/.config/wireguard/` or
`~/Downloads/`, then choose "Import WireGuard config" in the menu, or run:

```sh
vpn-menu import ~/Downloads/home.conf
```

NetworkManager stores the profile, including the private key, under
`/etc/NetworkManager/system-connections/` readable by root only. The original
file is moved to the trash so no plain copy of the key stays in your home.
The profile is named after the file. `wireguard-tools` provides `wg genkey`
for making your own keys.

## Tailscale

Tailscale's client is open source; its coordination server is a hosted
service. Headscale is the self-hosted alternative and works with the same
client. Install the client from Tailscale's Debian repository following
[their instructions](https://tailscale.com/kb/1031/install-linux), then rerun
setup, which grants your user operator rights so the menu can toggle it
without sudo:

```sh
sudo tailscale set --operator="$USER"
```

The first "Tailscale" choice opens the login page in the browser. Finish
signing in, then choose Tailscale again. The [firewall](firewall.md) drops
unsolicited incoming traffic, so peers reach this machine through Tailscale's
relays unless you allow UDP 41641 in.

## Checks

`vpn-menu status` prints the active tunnel names; the status panel shows them
on a VPN line. Offline tests cover listing, toggling, the fail-off teardown,
import with autoconnect off, login URL handling and session start.
