# Current state

Updated 2026-10-08. The goal is a fresh Debian 13 x86_64 installation configured
by `./scripts/setup`, without manual application theme edits or cleanup tailored
to the development VM. See [installation](installation.md) for the entry point
and [TODO](TODO.md) for remaining decisions and verification.

| Area | Current repository behavior |
| --- | --- |
| Remote entry point | `scripts/bootstrap` downloads a disposable checkout and removes it after setup or failure. Publication and fresh-machine verification remain pending. |
| Monitors | Fresh installs receive the saved laptop/LG profiles; existing local kanshi settings are preserved. |
| Shell | Prefix Up/Down (user-confirmed), history/completion autosuggestions, fzf Ctrl+R and pinned fzf-tab setup. Interactive history checks pass; plugin download and live Tab/previews remain unverified in the restricted session. See [shell](shell.md). |
| Full setup | Installs Debian packages, missing verified Yazi/Zen/font releases, pinned termfilechooser and fzf-tab, Neovim/plugins, user files, initial theme, groups, zsh login shell and APT policy. Existing apps/configs are reused where appropriate. |
| Dark defaults | GTK dark settings and desktop preference; explicit KeePassXC/OpenSnitch dark themes; app launchers reapply dark on startup. OpenSnitch theme support is an installed dependency. Zen requests dark content; websites must implement the preference. No desktop environment or global Qt theme plugin is required. |
| OpenSnitch | Setup provisions a daemon socket override for the installing user's UID; the Sway session starts the matching UI service. Existing daemon preferences and rules are preserved. Fresh-login collection still needs live verification. |
| Chromium | Installed from Debian by full setup, alongside Zen. |
| Python | IPython keeps its normal recommended dependencies. NumPy, Matplotlib and SciPy are explicit setup packages, as requested. |
| Passwords | Super+P opens/focuses/hides KeePassXC; browser integration and native messaging defaults are configured. Database creation/selection and Zen extension pairing remain interactive. See [passwords](passwords.md). |
| Upload/save picker | Zen uses XDG Desktop Portal → termfilechooser → Yazi in Foot. The picker leaves Sway fullscreen before opening, then restores it when closed unless another window is fullscreen. This is the desired behavior; preserve it. |
| General fullscreen windows | The popup guard exits fullscreen for new windows, floats/centers/focuses them and restores fullscreen after tracked windows close or leave. Explicit fullscreen actions take priority. See [window behavior](window-behavior.md). |
| Authentication prompts | Setup installs mate-polkit; Sway starts its agent directly in the desktop login session. The user confirmed the graphical authentication test works on 2026-10-08 after switching to direct Sway startup. |
| Sway/session | Session target starts the IPC monitor, popup guard, kanshi, Mako and update notifications. Local monitor layouts remain local. Live service health is separate from configuration. |
| Locking/clipboard | 30-minute idle lock and lock before suspend; no blanket fullscreen inhibition. Automatic clipboard history capture is disabled. KeePassXC defaults include five-minute idle locking and ten-second clipboard clearing. |
| Updates | Setup installs both APT policy files and enables Debian update timers. No automatic reboots or dependency/kernel removal. |
| Editor | Neovim/plugins and in-editor Markdown preview; optional development tools stay explicit. Browser Markdown preview is disabled. |
| Obsidian | Not installed by the repo. Setup does not uninstall it or perform development-VM cleanup. |
| Scope | Setup now activates the repository firewall; required incoming exceptions remain machine-specific. Encryption, firmware/drivers, monitor adjustments, VM networking/storage, hardware unlock and backup choices remain manual. Existing Incus/data are not targeted. |

## Verification

The user ran full setup on the development VM and rebooted. Read-only checks
confirmed required packages, installed user files, both APT policy files, shell,
group memberships, locale and portal backend files. The user confirmed Zen
uploads through Yazi and the fullscreen-exit/floating-picker behavior. These
checks precede later package/theme additions and do not establish a fresh-machine
installation of the final revision.

The current sandbox cannot certify live systemd/Sway state or apply privileged
package changes. Older host snapshots in audit documents are dated evidence,
not current service status. Fresh installation, final dark app appearance,
browser pairing/autofill, save/overwrite, screen sharing and hardware behavior
still need their respective live checks. See [testing](testing.md).
## System health — 2026-10-08

The report and user timer are installed. The user accepted the simplified tiles
and confirmed OpenSnitch collection after correcting the UI socket mismatch.
The report works without the source checkout. Installation receipts are recorded
by future successful installs; the previous installation has no receipt.

Session startup and picker-cancellation fixes are installed and covered by 25
passing command/portal tests. UPower is in the package list but still needs local
installation. Live timer behavior, ShellCheck, full journal access, SMART access
and fresh-install daemon socket configuration remain unverified. See
[system health](sysreport.md) and [TODO](TODO.md).
