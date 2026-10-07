## Shared glass switch

Use `glass-mode on`, `glass-mode off`, `glass-mode toggle`, or `glass-mode status`.
**Super+Ctrl+G** toggles the same setting. Glass is currently enabled.
Restart Zen and reopen terminals after switching; GTK applications may need reopening.
The command leaves running applications open and preserves the current color palette.

State lives in `~/.config/theme/glass-mode`. Theme templates render solid or glass
backgrounds from that state, so subsequent palette changes preserve the mode.
Zen uses a managed import in userChrome.css and startup preferences in user.js;
other custom CSS and preferences are retained. Real web pages remain opaque.
Browser tint is 8%, floating sidebar tint 38%, and search overlay tint 24%.
Desktop transparency has no compositor blur on standard Sway.

The older `transparency-trial` command is a snapshot rollback utility, not the mode
switch. Its original backups remain in `~/.local/state/linux-config/transparency-trial`.
Do not use that rollback for routine toggling; later edits can conflict with it.

# Transparency experiment

The optional `transparency-trial on` command enables a reversible desktop trial.
It updates both repository sources and installed configs, saving exact originals
under `~/.local/state/linux-config/transparency-trial/` before making changes.
Run `transparency-trial status` to check whether it is enabled.

- Zen's sidebar and toolbars use translucent tinted backgrounds with subtle
  borders. Text, icons and website content remain fully opaque.
- Sidebar backing outlines and the extra divider are removed. The collapsed
  search field is clear; its expanded panel uses a translucent blurred fill.
- Zen's explicit empty-tab placeholder is transparent. Only while
  `zen-has-empty-tab=true`, its content-browser surface is hidden and its backing
  cleared. Opening a real webpage restores the normal opaque content surface.
- GTK window backgrounds, headers, menus and dialogs use translucent fills.
  Application-specific themes may override some surfaces.
- Foot backgrounds use 72% opacity for newly opened terminals.
- Fuzzel launchers and pickers use 72% background opacity.
- Mako notifications and status popups use 72% background opacity.
- Theme templates are updated too, so switching palettes retains the experiment.
- A static woven wallpaper adds subtle texture and tonal variation behind the
  translucent surfaces. `wallpaper-texture` regenerates it from the current
  palette; the Sway theme target runs this hook after palette changes.

Standard Sway provides transparency without background blur. The woven wallpaper
makes the effect visible without animation. The desktop background and lock
screen remain opaque. Undo restores the original solid wallpaper configuration;
the generated PNG can remain on disk without affecting the desktop.
The compositor keeps all application windows at full opacity. Zen uses
`zen.widget.linux.transparency=true` and custom `userChrome.css`, while
`browser.tabs.allow_transparent_browser=false` preserves normal web pages.
The CSS source lives in `dotfiles/.config/zen/userChrome.css`; the trial copies
it into the default installation profile identified by `profiles.ini`.
Modern Zen paints workspace colors on `.zen-browser-generic-background`
pseudo-elements. The trial overrides those layers directly, including the
additional solid backing on `.zen-toolbar-background` in compact mode. Its
18% tint is controlled by `--glass-tint`. Compact overlays request browser-local
backdrop blur; this can blur website content behind a floating toolbar, but it
cannot blur desktop content outside the browser on standard Sway.
The compact overlay also adapts Natsumi's conditional webpage `clip-path`
workaround so Firefox can sample page pixels for backdrop blur. It applies only
while the floating sidebar or toolbar is visible. A local test on Zen 1.22.3b
confirmed that stripes behind the floating sidebar blur while the rest of the
page remains sharp and opaque. Natsumi's MIT notice is retained in
`docs/licenses/natsumi-MIT.txt`.
Close Zen before toggling the trial, then reopen it to load the configuration.
Only the three managed browser preferences are restored on undo; unrelated
browser preferences are preserved even after browsing during the trial.

Undo with:

```sh
transparency-trial off
```

This restores original files and resets application window opacity to 1.
Reopen terminals after enabling or disabling for their native background setting
to change. The command refuses to overwrite files edited since the trial began;
if a palette switch or another edit causes this, review the listed files and
saved originals before restoring manually. Backups remain available after undo.

References: [Sway opacity](https://man.archlinux.org/man/sway.5),
[Foot colors](https://man.archlinux.org/man/foot.ini.5.en), and
[Fuzzel RGBA colors](https://man.archlinux.org/man/fuzzel.ini.5).
Zen's [advanced preferences](https://docs.zen-browser.app/guides/about-config-flags)
document the separate browser UI and website transparency settings.
See also [modern Zen selector changes](https://github.com/catppuccin/zen-browser/pull/61)
and [SwayFX's compositor blur support](https://github.com/wlrfx/swayfx).

## Mod source review

[Transparent Zen](https://github.com/sameerasw/zen-themes/tree/main/TransparentZen)
provides CSS plus preference controls for UI and website transparency, tint,
animations and other changes. Its website transparency switches are unnecessary
for this trial. Installing it would overlap the custom CSS rather than unlock
compositor blur.

[Natsumi's sidebar module](https://github.com/Psychosoc1al/natsumi/blob/main/natsumi/modules/sidebars.css)
combines translucent fills, backdrop blur, borders and shadows with a conditional
clip on webpage surfaces. The inspected fork is archived and targets Zen 1.8.1;
the trial adapts only the small compositing workaround to current selectors,
instead of installing the complete historical layout and animations.
