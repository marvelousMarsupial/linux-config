"""Installation regression tests."""
from pathlib import Path
import tempfile
import unittest
from support import REPO, run


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)

    def install(self, *args):
        return run(str(REPO / 'scripts/install'), '--home', str(self.home), *args)

    def test_dry_run_backup_and_repeat(self):
        target = self.home / '.zshrc'
        target.write_text('original shell\n')
        self.assertEqual(self.install('--dry-run').returncode, 0)
        self.assertEqual(target.read_text(), 'original shell\n')
        self.assertFalse((self.home / '.local').exists())
        self.assertEqual(self.install().returncode, 0)
        self.assertEqual(target.read_bytes(), (REPO / 'dotfiles/.zshrc').read_bytes())
        backup = list((self.home / '.local/state/linux-config/backups').glob('*/.zshrc'))
        self.assertEqual(len(backup), 1)
        self.assertEqual(backup[0].read_text(), 'original shell\n')
        again = self.install()
        self.assertEqual(again.returncode, 0, again.stderr)
        self.assertIn('0 file(s) changed.', again.stdout)
        self.assertEqual((self.home / '.local/bin/theme').stat().st_mode & 0o777, 0o755)
        self.assertEqual(target.stat().st_mode & 0o777, 0o644)
        self.assertFalse(list(self.home.rglob('*.pyc')))
        self.assertFalse(list(self.home.rglob('__pycache__')))

    def test_terminal_install_skips_desktop_files(self):
        result = self.install('--terminal')
        self.assertEqual(result.returncode, 0, result.stderr)
        for rel in ('.zshrc', '.config/starship.toml', '.config/nvim/init.lua', '.config/yazi/yazi.toml', '.local/bin/bat'):
            self.assertTrue((self.home / rel).is_file(), rel)
        for rel in ('.config/sway/config', '.config/foot/foot.ini', '.local/bin/theme', '.config/systemd'):
            self.assertFalse((self.home / rel).exists(), rel)
        self.assertNotIn('theme apply', result.stdout)
        self.assertIn('terminal-install', (self.home / '.local/state/linux-config/install-info').read_text())

    def test_preserves_colors_monitors_and_plugin_lock(self):
        paths = ['.config/kanshi/config', '.config/sway/colors', '.config/foot/colors.ini',
                 '.config/nvim/lazy-lock.json', '.config/nvim/lua/plugins/colorscheme.lua']
        for rel in paths:
            p = self.home / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text('local customization\n')
        self.assertEqual(self.install().returncode, 0)
        for rel in paths:
            self.assertEqual((self.home / rel).read_text(), 'local customization\n')

    def test_browser_launcher_installs_without_touching_profiles(self):
        profile = self.home / '.mozilla/firefox/existing.default/prefs.js'
        profile.parent.mkdir(parents=True)
        profile.write_text('user_pref("unrelated.setting", true);\n')
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        launcher = self.home / '.local/bin/firefox'
        self.assertEqual(launcher.stat().st_mode & 0o777, 0o755)
        self.assertIn('GTK_USE_PORTAL=1', launcher.read_text())
        entry = (self.home / '.local/share/applications/firefox-esr.desktop').read_text()
        self.assertIn(f'Exec="{self.home}/.local/bin/firefox" %u', entry)
        # Browser prefs, policies and CSS come from the firefoxconfig package, not this repo.
        self.assertFalse((self.home / '.surfingkeys.js').exists())
        self.assertFalse(list(self.home.rglob('policies.json')))
        self.assertEqual(profile.read_text(), 'user_pref("unrelated.setting", true);\n')
        self.assertEqual(list((self.home / '.mozilla/firefox').iterdir()), [profile.parent])

    def test_backs_up_symlink_without_writing_through_it(self):
        original = self.home / 'original'
        original.write_text('leave me alone\n')
        (self.home / '.zshrc').symlink_to(original)
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(original.read_text(), 'leave me alone\n')
        self.assertFalse((self.home / '.zshrc').is_symlink())
        backup = next((self.home / '.local/state/linux-config/backups').glob('*/.zshrc'))
        self.assertTrue(backup.is_symlink())

    def test_retires_obsolete_files_with_backups(self):
        rel = '.tmux.conf'
        path = self.home / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('old plugin')
        self.assertEqual(self.install('--dry-run').returncode, 0)
        self.assertTrue(path.exists())
        self.assertEqual(self.install().returncode, 0)
        self.assertFalse(path.exists())
        saved = list((self.home / '.local/state/linux-config/backups').glob('*/' + rel))
        self.assertEqual(saved[0].read_text(), 'old plugin')

    def test_refuses_directory_collision(self):
        (self.home / '.bashrc').mkdir()
        result = self.install()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Refusing to replace directory', result.stderr)

    def test_backup_parent_symlink_cannot_escape_home(self):
        with tempfile.TemporaryDirectory() as outside:
            state = self.home / '.local/state/linux-config'
            state.mkdir(parents=True)
            (state / 'backups').symlink_to(outside, target_is_directory=True)
            (self.home / '.zshrc').write_text('original')
            result = self.install()
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(list(Path(outside).iterdir()), [])
            self.assertEqual((self.home / '.zshrc').read_text(), 'original')

    def test_missing_destination_home_is_created(self):
        self.home = self.home / 'new home'
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((self.home / '.config/nvim/init.lua').is_file())

    def test_mode_drift_is_repaired_and_backed_up(self):
        self.assertEqual(self.install().returncode, 0)
        script = self.home / '.local/bin/theme'
        script.chmod(0o644)
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(script.stat().st_mode & 0o777, 0o755)
        backups = list((self.home / '.local/state/linux-config/backups').glob('*/.local/bin/theme'))
        self.assertEqual(len(backups), 1)
        self.assertEqual(backups[0].stat().st_mode & 0o777, 0o644)

    def test_app_dark_themes_merge_without_replacing_existing_settings(self):
        fixtures = {
            '.config/keepassxc/keepassxc.ini': ('[General]\nLastDatabases=private.kdbx\n[Browser]\nEnabled=true\n[GUI]\nApplicationTheme=light\nHideToolbar=true\n', 'ApplicationTheme=dark'),
            '.config/opensnitch/settings.conf': ('[global]\ndefault_action=1\ntheme=light_blue.xml\n[statsDialog]\ngeometry=@ByteArray(\\0\\x1)\n', 'theme=dark_amber.xml'),
        }
        for rel, (original, expected) in fixtures.items():
            path = self.home / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(original)
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        for rel, (original, expected) in fixtures.items():
            path = self.home / rel
            text = path.read_text()
            for line in original.splitlines():
                if not line.startswith(('ApplicationTheme=', 'theme=')):
                    self.assertIn(line, text)
            self.assertIn(expected, text)
            if rel.endswith('settings.conf'):
                self.assertIn(f'file={self.home}/.local/share/opensnitch.db', text)
                self.assertIn('type=1', text)
                self.assertNotIn('/home/user', text)
            self.assertEqual(path.stat().st_mode & 0o777, 0o600)
            backup = next((self.home / '.local/state/linux-config/backups').glob('*/' + rel))
            self.assertEqual(backup.read_text(), original)
        again = self.install()
        self.assertIn('0 file(s) changed.', again.stdout)
