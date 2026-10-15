"""Themes regression tests."""
import json
import os
from pathlib import Path
import re
import shutil
import tempfile
import tomllib
import unittest
from support import REPO, run


class ThemeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)
        self.root = self.home / '.config/theme'
        shutil.copytree(REPO / 'dotfiles/.config/theme', self.root)
        config = self.root / 'config.toml'
        # Prevent temporary-home tests from reloading the real user's applications.
        config.write_text(re.sub(r'^hook = .*\n', '', config.read_text(), flags=re.M))
        self.env = dict(os.environ, HOME=str(self.home))

    def theme(self, *args):
        return run(str(REPO / 'dotfiles/.local/bin/theme'), *args, env=self.env)

    def test_every_scheme_applies_and_parses(self):
        schemes = sorted((self.root / 'schemes').glob('*.yaml'))
        self.assertEqual(len(schemes), 6)
        for scheme in schemes:
            with self.subTest(scheme=scheme.stem):
                result = self.theme('apply', scheme.stem)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(self.theme('current').stdout.strip(), scheme.stem)
                first = (self.home / '.config/foot/colors.ini').read_text()
                sway = (self.home / '.config/sway/colors').read_text()
                foreground = re.search(r'^foreground=(\w+)', first, re.M)[1]
                background = re.search(r'^background=(\w+)', first, re.M)[1]
                self.assertRegex(foreground, r'^[0-9a-fA-F]{6}$')
                self.assertEqual(background, re.search(r'^set \$base01 #(\w+)', sway, re.M)[1])
                self.assertNotEqual(background, re.search(r'^set \$base00 #(\w+)', sway, re.M)[1])
                yazi = tomllib.loads((self.home / '.config/yazi/theme.toml').read_text())
                for rule in yazi.get('filetype', {}).get('rules', []):
                    # Yazi requires url or mime; the older name key stops it from starting.
                    self.assertTrue({'url', 'mime'} & rule.keys(), rule)
                if shutil.which('foot'):
                    config = self.home / '.config/foot/foot.ini'
                    config.write_text(f'include={self.home}/.config/foot/colors.ini\n')
                    checked = run('foot', '--check-config', '--config', str(config), env=self.env)
                    self.assertEqual(checked.returncode, 0, checked.stderr)
                if shutil.which('fuzzel'):
                    config = self.home / '.config/fuzzel/fuzzel.ini'
                    config.write_text(f'include={self.home}/.config/fuzzel/colors.ini\n')
                    checked = run('fuzzel', '--check-config', '--config', str(config), env=self.env)
                    self.assertEqual(checked.returncode, 0, checked.stderr)
                if shutil.which('nvim'):
                    lua = self.home / '.config/nvim/lua/plugins/colorscheme.lua'
                    checked = run('nvim', '--headless', '-u', 'NONE',
                                  '-c', f'lua assert(loadfile({json.dumps(str(lua))}))', '+qa', env=self.env)
                    self.assertEqual(checked.returncode, 0, checked.stderr)
                    self.assertNotIn('Error', checked.stderr)

    def test_preview_does_not_write_outputs(self):
        self.assertEqual(self.theme('preview', '1').returncode, 0)
        self.assertFalse((self.home / '.cache/theme/current').exists())
        self.assertFalse((self.home / '.config/sway/colors').exists())

    def test_invalid_scheme_fails_without_writing(self):
        self.assertNotEqual(self.theme('apply', '../../etc/passwd').returncode, 0)
        self.assertNotEqual(self.theme('apply', '99').returncode, 0)
        self.assertFalse((self.home / '.config/sway').exists())

    def test_bad_template_does_not_partially_apply(self):
        self.assertEqual(self.theme('apply', 'kanagawa').returncode, 0)
        target = self.home / '.config/foot/colors.ini'
        before = target.read_bytes()
        (self.root / 'templates/gtk4.css.j2').write_text('{{ missing_variable }}')
        self.assertNotEqual(self.theme('apply', 'summer-dusk').returncode, 0)
        self.assertEqual(target.read_bytes(), before)
        self.assertEqual(self.theme('current').stdout.strip(), 'kanagawa')

    def test_invalid_palette_does_not_write(self):
        path = self.root / 'schemes/kanagawa.yaml'
        path.write_text(path.read_text().replace('1f1f28', 'invalid'))
        self.assertNotEqual(self.theme('apply', 'kanagawa').returncode, 0)
        self.assertFalse((self.home / '.config/sway/colors').exists())

    def test_glass_wallpaper_tiles_dots_by_default(self):
        (self.root / 'glass-mode').write_text('on\n')
        self.assertEqual(self.theme('apply', 'summer-dusk').returncode, 0)
        sway = (self.home / '.config/sway/colors').read_text()
        self.assertRegex(sway, r'(?m)^output \* bg ~/.local/share/wallpapers/glass-texture.png tile ')
        (self.root / 'wallpaper-style').write_text('frost\n')
        self.assertEqual(self.theme('apply', 'summer-dusk').returncode, 0)
        sway = (self.home / '.config/sway/colors').read_text()
        self.assertRegex(sway, r'(?m)^output \* bg ~/.local/share/wallpapers/glass-texture.png fill ')
        (self.root / 'glass-mode').write_text('off\n')
        self.assertEqual(self.theme('apply', 'summer-dusk').returncode, 0)
        self.assertIn('output * bg $base00 solid_color', (self.home / '.config/sway/colors').read_text())


class YaziFlavorTests(unittest.TestCase):
    def test_flavor_filetype_rules_use_url_or_mime(self):
        flavors = sorted((REPO / 'dotfiles/.config/yazi/flavors').glob('*/flavor.toml'))
        self.assertTrue(flavors)
        for flavor in flavors:
            with self.subTest(flavor=flavor.parent.name):
                for rule in tomllib.loads(flavor.read_text())['filetype']['rules']:
                    self.assertTrue({'url', 'mime'} & rule.keys(), rule)
