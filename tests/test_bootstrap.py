"""Desktop setup boundaries using local files and mocked privileged/network actions."""
import hashlib
import importlib.machinery
import importlib.util
import io
import json
import os
import shutil
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile
from support import REPO, run

loader = importlib.machinery.SourceFileLoader('desktop_setup', str(REPO / 'scripts/setup'))
spec = importlib.util.spec_from_loader('desktop_setup', loader)
setup = importlib.util.module_from_spec(spec)
loader.exec_module(setup)


class BootstrapTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)

    def test_opensnitch_provisions_matching_socket_and_preserves_local_policy(self):
        commands = []
        def local_path(value):
            value = str(value)
            return self.home / value.lstrip('/') if value.startswith(('/etc/', '/var/')) else Path(value)
        def command(*args, **kwargs):
            commands.append(tuple(map(str, args)))
            if args[1] == 'install':
                if '-d' in args:
                    Path(args[-1]).mkdir(parents=True, exist_ok=True)
                else:
                    Path(args[-1]).parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(args[-2], args[-1])
            elif args[1] == 'cp':
                shutil.copyfile(args[-2], args[-1])
        policy = local_path('/etc/opensnitchd/default-config.json')
        policy.parent.mkdir(parents=True)
        policy.write_text('{"DefaultAction":"deny","Server":{"Address":"unix:///tmp/osui.sock"}}')
        original = policy.read_bytes()
        destination = local_path('/etc/systemd/system/opensnitch.service.d/90-linux-config.conf')
        with patch.object(setup, 'Path', side_effect=local_path), \
             patch.object(setup.os, 'getuid', return_value=1234), \
             patch.object(setup, 'run', side_effect=command):
            setup.opensnitch(False)
            service = (REPO / 'dotfiles/.config/systemd/user/opensnitch-ui.service').read_text()
            socket = 'unix:///run/user/1234/opensnitch/osui.sock'
            self.assertIn('-ui-socket ' + socket, destination.read_text())
            self.assertIn('--socket ' + socket, service.replace('%t', '/run/user/1234'))
            self.assertIn('opensnitch-ui.service', (REPO / 'dotfiles/.config/systemd/user/sway-session.target').read_text())
            self.assertIn(('sudo', 'systemctl', 'restart', 'opensnitch.service'), commands)
            commands.clear()
            setup.opensnitch(False)
            self.assertEqual(commands, [('sudo', 'systemctl', 'enable', '--now', 'opensnitch.service')])
            destination.write_text('old override\n')
            setup.opensnitch(False)
        backups = list((self.home / 'var/backups').rglob('90-linux-config.conf'))
        self.assertEqual(len(backups), 1)
        self.assertEqual(backups[0].read_text(), 'old override\n')
        self.assertEqual(policy.read_bytes(), original)

    def test_dry_run_does_not_write_home_or_call_sudo(self):
        binary = self.home / 'bin'
        binary.mkdir()
        sudo = binary / 'sudo'
        sudo.write_text('#!/bin/sh\necho unexpected-sudo >&2\nexit 99\n')
        sudo.chmod(0o755)
        result = run(str(REPO / 'scripts/setup'), '--dry-run',
                     env=dict(os.environ, HOME=str(self.home), PATH=f'{binary}:/usr/bin:/bin'))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('Plan complete.', result.stdout)
        self.assertIn('zen-browser/desktop', result.stdout)
        self.assertIn('--skip-apt', result.stdout)
        self.assertIn('configure OpenSnitch UI socket:', result.stdout)
        self.assertIn('enable --now opensnitch.service', result.stdout)
        self.assertFalse((self.home / '.local').exists())
        self.assertNotIn('unexpected-sudo', result.stderr)

    def test_verified_download_and_rejected_digest(self):
        payload = b'release fixture'
        asset = {'name': 'fixture.zip', 'digest': 'sha256:' + hashlib.sha256(payload).hexdigest(),
                 'browser_download_url': 'https://github.com/example/app/releases/download/v1/fixture.zip'}
        def response(url):
            return io.BytesIO(json.dumps({'tag_name': 'v1', 'assets': [asset]}).encode()
                              if 'api.github.com' in url else payload)
        destination = self.home / 'download'
        with patch.object(setup, 'request', side_effect=response):
            setup.release_asset('example/app', 'fixture.zip', destination)
            self.assertEqual(destination.read_bytes(), payload)
            asset['digest'] = 'sha256:' + '0' * 64
            with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
                setup.release_asset('example/app', 'fixture.zip', destination)
            asset['digest'] = None
            with self.assertRaisesRegex(ValueError, 'No published SHA-256'):
                setup.release_asset('example/app', 'fixture.zip', destination)

    def test_archive_traversal_and_symlinks_rejected_before_extraction(self):
        archive = self.home / 'archive'
        with zipfile.ZipFile(archive, 'w') as output:
            output.writestr('../outside', 'bad')
        with self.assertRaisesRegex(ValueError, 'Unsafe'):
            setup.unpack(archive, self.home / 'zip')
        with tarfile.open(archive, 'w') as output:
            item = tarfile.TarInfo('link')
            item.type = tarfile.SYMTYPE
            item.linkname = '/etc/passwd'
            output.addfile(item)
        with self.assertRaisesRegex(ValueError, 'Unsafe'):
            setup.unpack(archive, self.home / 'tar')
        self.assertFalse((self.home / 'outside').exists())

    def test_installed_applications_are_preserved(self):
        zen = self.home / '.local/zen/zen'
        zen.parent.mkdir(parents=True)
        zen.write_text('existing browser')
        with patch.object(setup.shutil, 'which', return_value='/bin/existing'), \
             patch.object(setup.subprocess, 'run') as command, \
             patch.object(setup, 'release_asset') as download:
            command.return_value.stdout = 'CaskaydiaCove Nerd Font Mono'
            setup.applications(self.home, False)
            download.assert_not_called()
        self.assertEqual(zen.read_text(), 'existing browser')

    def test_failed_replacement_restores_previous_installation(self):
        dest = self.home / 'installed'
        dest.write_text('previous')
        backup = self.home / 'backup'
        with self.assertRaises(OSError):
            setup.replace(self.home / 'missing', dest, backup)
        self.assertEqual(dest.read_text(), 'previous')
        self.assertFalse(backup.exists())

    def test_fresh_home_installs_all_missing_applications(self):
        def download(repository, name, destination):
            if repository == 'zen-browser/desktop':
                with tarfile.open(destination, 'w') as archive:
                    item = tarfile.TarInfo('zen/zen')
                    payload = b'#!/bin/sh\nexit 0\n'
                    item.size, item.mode = len(payload), 0o755
                    archive.addfile(item, io.BytesIO(payload))
            else:
                with zipfile.ZipFile(destination, 'w') as archive:
                    if repository == 'sxyazi/yazi':
                        archive.writestr('yazi-release/yazi', '#!/bin/sh\nexit 0\n')
                        archive.writestr('yazi-release/ya', '#!/bin/sh\nexit 0\n')
                    else:
                        archive.writestr('CaskaydiaCoveNerdFont-Regular.ttf', 'fixture')

        with patch.object(setup.shutil, 'which', return_value=None), \
             patch.object(setup, 'release_asset', side_effect=download) as fetch, \
             patch.object(setup, 'run'):
            setup.applications(self.home, False)
            self.assertEqual(fetch.call_count, 3)
        for executable in ('.local/bin/yazi', '.local/bin/ya', '.local/zen/zen'):
            path = self.home / executable
            self.assertTrue(path.is_file())
            self.assertEqual(path.stat().st_mode & 0o777, 0o755)
        self.assertTrue(list((self.home / '.local/share/fonts').rglob('*.ttf')))
        self.assertEqual(list((self.home / '.local').glob('linux-config-*')), [])

    def test_other_username_is_rendered_and_repeat_install_is_clean(self):
        result = run(str(REPO / 'scripts/install'), '--home', str(self.home))
        self.assertEqual(result.returncode, 0, result.stderr)
        for rel in ('.config/sway/config', '.config/theme/config.toml', '.local/share/applications/zen.desktop'):
            content = (self.home / rel).read_text()
            self.assertNotIn('/home/august', content)
            self.assertIn(str(self.home), content)
        result = run(str(REPO / 'scripts/install'), '--home', str(self.home))
        self.assertIn('0 file(s) changed.', result.stdout)
