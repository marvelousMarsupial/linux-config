# KeePassXC and Firefox

[Documentation index](README.md)

The intended workflow is Super+P to open/unlock KeePassXC, then use the
KeePassXC-Browser extension in Firefox to fill logins. Super+P hides a focused
KeePassXC window in the Sway scratchpad and shows it again on the next press.
Hiding does not lock or quit KeePassXC. A window on another workspace is focused
there. New KeePassXC windows float and are centered.

## First use

1. Reload Sway with Super+Shift+C, then press Super+P.
2. Open your existing `.kdbx` file, or create a database in KeePassXC. Enter the
   master password in the application. Keep the database and master password out
   of this repository and chat. KeePassXC remembers the last database.
3. Install [KeePassXC-Browser for Firefox](https://addons.mozilla.org/firefox/addon/keepassxc-browser/)
   in Firefox; the firefoxconfig package installs it by policy. Pin its
   toolbar button for convenient access.
4. With the database unlocked, click the extension and choose **Connect**. Give
   the connection a recognizable name such as `Firefox on Debian` and approve it in
   KeePassXC. This pairing is stored in your database.
5. Test with a disposable login whose URL matches the test website. Approve the
   initial request in KeePassXC, then use the extension's fill button. The usual
   Firefox extension shortcut is Ctrl+Shift+U; check the extension shortcuts if
   it conflicts. Keep automatic submission off so you can review the filled form.

Browser integration uses `/usr/bin/keepassxc-proxy` and the Firefox native
messaging manifest in `~/.mozilla/native-messaging-hosts/`. The extension talks to the local app; the app must be running
and the database unlocked. Pairing and first-site approvals are interactive.

## Defaults and existing settings

The repo seeds the explicit Dark application theme, browser integration, automatic saves, a backup before save,
a five-minute inactivity lock, lock on screen lock/user switch, and clipboard
clearing after ten seconds. Minimizing/hiding does not immediately lock the
database. Browser access and update approvals remain enabled. Existing KeePassXC
configuration is preserved by `scripts/install`, including settings and database
history, except that the application theme is set to Dark. The repo launcher also
reapplies Dark before each launch; no manual theme configuration is needed.
Already-running windows update on their next fresh launch. Change other existing
settings in Tools → Settings.

The `.old.kdbx` save backup is recovery from the previous save, not an independent
backup. Arrange a separate encrypted database backup and test recovery. Second
factors and hardware unlock are separate decisions.

The current VM has Debian KeePassXC 2.7.10. Do not assume the roadmap's planned
Linux fingerprint quick unlock is available: the official stable download checked
on 2026-10-08 is 2.7.12. Fingerprint support needs its own version/hardware check.

## Verification

Offline tests cover first launch, focus, scratchpad show/hide, invalid IPC, and
preserving existing application settings. Live database creation/opening,
extension installation/pairing, autofill, fullscreen visibility and lock behavior
must be checked in the actual Sway session. In particular, confirm Sway's lock
notification reaches KeePassXC; the enabled application setting alone does not
prove that integration works.

Sources: [KeePassXC browser setup](https://keepassxc.org/docs/KeePassXC_GettingStarted#_setup_browser_integration),
[upstream native messaging implementation](https://github.com/keepassxreboot/keepassxc/blob/2.7.10/src/browser/NativeMessageInstaller.cpp),
[official downloads](https://keepassxc.org/download/).
