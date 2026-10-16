# Manual setup checklist

[Documentation index](README.md)

After `./scripts/setup`. Follow the links only when you need details.

- [ ] **Restart:** reboot or log out and back in, then fully restart Firefox.
- [ ] **Shell:** open a new terminal; test prefix Up/Down, Ctrl+R, grey
  suggestions, Tab selection and cancel, and directory/file previews.
  [Shell](shell.md)
- [ ] **Browser:** check the extensions in `about:addons` and the policies in
  `about:policies`, then follow the firefoxconfig README for Surfingkeys and
  the other first-start steps. [Browser](browser.md)
- [ ] **KeePassXC:** open or create your database, connect the browser
  extension, approve pairing, and test filling a login. [Passwords](passwords.md)
- [ ] **Monitors:** adjust `~/.config/kanshi/config` for this machine. Put
  machine-specific Sway settings in `~/.config/sway/local.conf`.
- [ ] **Desktop checks:** test file upload, save and cancel, screen sharing,
  locking, suspend and resume, and graphical password prompts.
  [Portals](portals.md), [systemd](systemd.md)
- [ ] **Security checks:** test OpenSnitch prompts, inspect the health report,
  and verify automatic updates. [Health](sysreport.md), [updates](updates.md)
- [ ] **Firewall:** setup activates the policy; verify required access, VM
  connectivity and filtering after reboot. [Firewall](firewall.md)

Only when needed:

- [ ] **Popup guard updated:** run `systemctl --user restart sway-popup-guard.service`
  or log out and back in. [Window behavior](window-behavior.md)
- [ ] **Backups:** choose target and password, test backup and restore, enable
  the timer. [Backups](backups.md)
- [ ] **Laptop:** test audio, volume and brightness keys, Wi-Fi and lid suspend;
  configure firmware/drivers and optional fingerprint or security key.
  [Hardware tasks](TODO.md#hardware-and-later-laptop-setup)
- [ ] **Machine choices:** disk encryption, boot/login behavior, optional lid
  policy, and VM networks/storage. [Installation](installation.md), [systemd](systemd.md)
- [ ] **Config-only install:** install missing apps, render your theme, reload
  user units, and activate needed services.
  [Installation](installation.md#config-only-installation)

Project work and deferred features live in [TODO](TODO.md).
