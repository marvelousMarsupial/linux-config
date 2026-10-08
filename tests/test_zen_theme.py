"""Desktop palettes in Zen, preserving glass and user profile customizations."""
import os
from pathlib import Path
import re
import shutil
import tempfile
import unittest
from support import REPO, load_script, run


class ZenThemeTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.home = Path(temporary.name)
        self.env = dict(os.environ, HOME=str(self.home))
        self.theme = REPO / 'dotfiles/.local/bin/theme'
        self.sync = load_script('zen-theme-sync')
        root = self.home / '.config/theme'
        shutil.copytree(REPO / 'dotfiles/.config/theme', root)
        config = root / 'config.toml'
        config.write_text(re.sub(r'^hook = .*\n', '', config.read_text(), flags=re.M))
        shutil.copytree(REPO / 'dotfiles/.config/zen', self.home / '.config/zen')
        zen = self.home / '.zen'
        zen.mkdir()
        (zen / 'profiles.ini').write_text(
            '[Profile0]\nPath=other.profile\nDefault=1\n\n'
            '[InstallFixture]\nDefault=default.profile\n')
        self.profile = zen / 'default.profile'
        self.profile.mkdir()
        other = zen / 'other.profile'
        other.mkdir()
        (other / 'user.js').write_text('// Leave this profile alone\n')

    def render(self, scheme='kanagawa'):
        result = run(str(self.theme), 'apply', scheme, env=self.env)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_all_schemes_update_default_profile_from_desktop_palette(self):
        import yaml
        for scheme in sorted((self.home / '.config/theme/schemes').glob('*.yaml')):
            with self.subTest(scheme=scheme.stem):
                self.render(scheme.stem)
                self.sync.sync(self.home)
                palette = yaml.safe_load(scheme.read_text())
                chrome = self.profile / 'chrome'
                css = (chrome / 'system-theme.css').read_text()
                for variable, slot in [('zen-primary-color', 'base0D'),
                                       ('zen-colors-primary', 'base01'),
                                       ('zen-colors-secondary', 'base02'),
                                       ('zen-colors-tertiary', 'base00'),
                                       ('toolbox-textcolor', 'base05'),
                                       ('urlbar-placeholder-color', 'base04'),
                                       ('panel-background', 'base01')]:
                    self.assertIn(f'--{variable}: #{palette[slot]} !important;', css)
                rgb = ', '.join(str(int(palette['base0D'][i:i + 2], 16)) for i in (0, 2, 4))
                self.assertIn(f'--glass-accent-rgb: {rgb};', (chrome / 'glass-palette.css').read_text())
                self.assertEqual((chrome / 'glass-chrome.css').read_text(),
                                 (self.home / '.config/zen/userChrome.css').read_text())
                self.assertFalse((self.home / '.zen/other.profile/chrome').exists())

    def test_import_precedes_glass_and_preserves_custom_css_and_prefs(self):
        self.render()
        chrome = self.profile / 'chrome'
        chrome.mkdir()
        original = ('/* glass-mode:begin */\n@import "glass-chrome.css";\n'
                    '/* glass-mode:end */\n#custom-rule { color: red; }\n')
        (chrome / 'userChrome.css').write_text(original)
        prefs = self.profile / 'user.js'
        prefs.write_text('user_pref("some.custom.pref", 42);\n'
                         'user_pref("zen.widget.linux.transparency", true);\n'
                         'user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", false);\n')
        self.sync.sync(self.home)
        first = (chrome / 'userChrome.css').read_text()
        self.sync.sync(self.home)
        self.assertEqual((chrome / 'userChrome.css').read_text(), first)
        self.assertTrue(first.endswith(original))
        self.assertLess(first.index('@import "system-theme.css"'), first.index('@import "glass-chrome.css"'))
        self.assertEqual(first.count('@import "system-theme.css"'), 1)
        text = prefs.read_text()
        self.assertIn('user_pref("some.custom.pref", 42);', text)
        self.assertIn('user_pref("zen.widget.linux.transparency", true);', text)
        self.assertIn('user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);', text)
        self.assertEqual(text.count('toolkit.legacyUserProfileCustomizations.stylesheets'), 1)
        self.assertEqual(prefs.stat().st_mode & 0o777, 0o600)

    def test_solid_mode_gets_theme_without_enabling_glass(self):
        self.render()
        self.sync.sync(self.home)
        css = (self.profile / 'chrome/userChrome.css').read_text()
        self.assertIn('@import "system-theme.css";', css)
        self.assertNotIn('@import "glass-chrome.css";', css)
        self.assertNotIn('zen.widget.linux.transparency', (self.profile / 'user.js').read_text())

    def test_missing_browser_is_a_noop(self):
        (self.home / '.zen/profiles.ini').unlink()
        self.sync.sync(self.home)
        self.assertFalse((self.profile / 'chrome').exists())

    def test_external_profile_and_chrome_symlinks_are_rejected(self):
        zen = self.home / '.zen'
        (zen / 'profiles.ini').write_text('[InstallFixture]\nDefault=../outside\n')
        (self.home / 'outside').mkdir()
        with self.assertRaisesRegex(ValueError, 'Invalid default Zen profile'):
            self.sync.sync(self.home)
        (zen / 'profiles.ini').write_text('[InstallFixture]\nDefault=default.profile\n')
        (self.profile / 'chrome').symlink_to(self.home / 'outside', target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'Invalid Zen chrome directory'):
            self.sync.sync(self.home)

    def test_missing_rendered_source_does_not_change_profile(self):
        self.render()
        (self.home / '.config/zen/system-theme.css').unlink()
        with self.assertRaises(FileNotFoundError):
            self.sync.sync(self.home)
        self.assertFalse((self.profile / 'chrome').exists())

    def test_glass_toggle_keeps_system_theme_import_in_correct_order(self):
        self.render()
        self.sync.sync(self.home)
        installed = self.home / '.local/bin'
        installed.mkdir(parents=True)
        shutil.copy2(self.theme, installed / 'theme')
        notify = installed / 'notify-send'
        notify.write_text('#!/bin/sh\nexit 0\n')
        notify.chmod(0o755)
        config = self.home / '.config/theme/config.toml'
        config.write_text(config.read_text().rstrip() + '\nhook = "' +
                          str(REPO / 'dotfiles/.local/bin/zen-theme-sync') + '"\n')
        env = dict(self.env, PATH=str(installed) + ':' + os.environ['PATH'])
        for action in ('on', 'off', 'on'):
            with self.subTest(action=action):
                result = run(str(REPO / 'dotfiles/.local/bin/glass-mode'), action, env=env)
                self.assertEqual(result.returncode, 0, result.stderr)
                css = (self.profile / 'chrome/userChrome.css').read_text()
                self.assertEqual(css.count('@import "system-theme.css";'), 1)
                if action == 'on':
                    self.assertLess(css.index('@import "system-theme.css"'),
                                    css.index('@import "glass-chrome.css"'))
                else:
                    self.assertNotIn('@import "glass-chrome.css"', css)
