"""Security regression tests."""
import os
from pathlib import Path
import tempfile
import unittest
from support import REPO, run


class SecurityAuditTests(unittest.TestCase):
    def test_runtime_environment_file_is_data_not_shell(self):
        rc = (REPO / 'dotfiles/.zshrc').read_text()
        block = rc[rc.index('# D-Bus session'):rc.index('# Plugins')]
        with tempfile.TemporaryDirectory() as d:
            runtime = Path(d)
            marker = runtime / 'executed'
            source = runtime / 'dbus-env'
            source.write_text(f'touch "{marker}"\nexport DBUS_SESSION_BUS_ADDRESS="unix:path=/tmp/$(touch {marker})"\n')
            env = dict(os.environ, XDG_RUNTIME_DIR=d, DBUS_SESSION_BUS_ADDRESS='unchanged')
            result = run('zsh', '-f', '-c', block + '\nprint -r -- "$DBUS_SESSION_BUS_ADDRESS"', env=env)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.strip(), 'unchanged')
            self.assertFalse(marker.exists())
            source.write_text("export DBUS_SESSION_BUS_ADDRESS='unix:path=/run/user/1000/bus'\n")
            result = run('zsh', '-f', '-c', block + '\nprint -r -- "$DBUS_SESSION_BUS_ADDRESS"', env=env)
            self.assertEqual(result.stdout.strip(), 'unix:path=/run/user/1000/bus')
            source.unlink()
            target = runtime / 'target'
            target.write_text("export DBUS_SESSION_BUS_ADDRESS='unix:path=/wrong'\n")
            source.symlink_to(target)
            result = run('zsh', '-f', '-c', block + '\nprint -r -- "$DBUS_SESSION_BUS_ADDRESS"', env=env)
            self.assertEqual(result.stdout.strip(), 'unchanged')

    def test_install_backups_private_even_with_shared_umask(self):
        with tempfile.TemporaryDirectory() as d:
            home = Path(d)
            (home / '.zshrc').write_text('original')
            result = run('sh', '-c', 'umask 002; exec "$1" --home "$2"', 'sh', str(REPO / 'scripts/install'), d)
            self.assertEqual(result.returncode, 0, result.stderr)
            backups = home / '.local/state/linux-config/backups'
            self.assertEqual(backups.stat().st_mode & 0o777, 0o700)
            self.assertTrue(all(p.stat().st_mode & 0o077 == 0 for p in backups.iterdir()))

    def test_download_checksum_failure_prevents_extraction_and_execution(self):
        with tempfile.TemporaryDirectory() as d:
            home = Path(d)
            tools = home / '.local/bin'
            tools.mkdir(parents=True)
            for name, body in {
                'nvim': 'exit 1',
                'curl': 'while [ "$#" -gt 0 ]; do if [ "$1" = -o ]; then shift; printf unchecked > "$1"; exit 0; fi; shift; done; exit 1',
                'tar': 'touch "$HOME/tar-ran"; exit 1',
            }.items():
                p = tools / name
                p.write_text('#!/bin/sh\n' + body + '\n')
                p.chmod(0o755)
            env = dict(os.environ, HOME=d, PATH=f'{tools}:/usr/bin:/bin')
            env.pop('NVIM_VERSION', None)
            env.pop('NVIM_SHA256', None)
            result = run(str(REPO / 'scripts/setup-editor'), '--skip-apt', env=env)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('checksum failed', result.stderr)
            self.assertFalse((home / 'tar-ran').exists())
            self.assertFalse((home / '.local/nvim').exists())

    def test_editor_refuses_modelines_and_public_preview(self):
        options = (REPO / 'dotfiles/.config/nvim/lua/config/options.lua').read_text()
        self.assertIn('opt.modeline = false', options)
        self.assertIn('opt.exrc = false', options)
        preview = (REPO / 'dotfiles/.config/nvim/lua/plugins/markdown.lua').read_text()
        self.assertIn('mkdp_open_to_the_world = 0', preview)
        self.assertIn('mkdp_auto_close = 1', preview)
        self.assertNotIn('mkdp#util#install', preview)
