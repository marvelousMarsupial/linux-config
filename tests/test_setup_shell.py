"""Pinned plugin installation must preserve working copies on download failure."""
from pathlib import Path
import runpy
import tempfile
import unittest
from unittest.mock import patch
from support import REPO


class SetupShellTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)
        self.module = runpy.run_path(str(REPO / 'scripts/setup-shell'))
        self.install = self.module['install']
        self.destination = self.home / '.local/share/zsh/fzf-tab'

    def test_dry_run_makes_no_files(self):
        self.install(self.home, dry=True)
        self.assertEqual(list(self.home.iterdir()), [])

    def test_install_replaces_with_backup_and_repeat_skips_network(self):
        self.destination.mkdir(parents=True)
        (self.destination / 'local-file').write_text('original')

        def command(*args, **kwargs):
            if args[:2] == ('git', 'clone'):
                source = args[-1]
                source.mkdir()
                (source / 'fzf-tab.plugin.zsh').write_text('# fixture\n')

        with patch.dict(self.module['helpers'], run=command), patch(
                'subprocess.check_output', return_value=self.module['COMMIT'] + '\n'):
            self.install(self.home)
        self.assertTrue((self.destination / 'fzf-tab.plugin.zsh').is_file())
        backups = list((self.home / '.local/state/linux-config/backups').glob('*/fzf-tab/local-file'))
        self.assertEqual(len(backups), 1)
        self.assertEqual(backups[0].read_text(), 'original')
        with patch.dict(self.module['helpers'], run=lambda *a, **k: self.fail('unexpected network')):
            self.install(self.home)

    def test_wrong_commit_keeps_existing_plugin(self):
        self.destination.mkdir(parents=True)
        marker = self.destination / '.linux-config-commit'
        marker.write_text('old revision\n')
        with patch.dict(self.module['helpers'], run=lambda *a, **k: None), patch(
                'subprocess.check_output', return_value='wrong commit\n'):
            with self.assertRaisesRegex(ValueError, 'pinned commit'):
                self.install(self.home)
        self.assertEqual(marker.read_text(), 'old revision\n')
        self.assertFalse(list(self.destination.parent.glob('.fzf-tab-*')))
