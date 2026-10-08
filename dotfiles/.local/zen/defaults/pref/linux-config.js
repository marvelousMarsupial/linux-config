// Application defaults load before the first profile starts. Keep them editable
// through about:config; profile theme sync reapplies the desktop's dark colors.
pref("zen.welcome-screen.seen", true);
// Sway manages windows; omit Zen's redundant caption controls/native title bar.
pref("browser.tabs.inTitlebar", 1);
pref("zen.view.experimental-no-window-controls", true);
pref("ui.systemUsesDarkTheme", 1);
pref("browser.theme.toolbar-theme", 0);
pref("browser.theme.content-theme", 0);
pref("layout.css.prefers-color-scheme.content-override", 0);
