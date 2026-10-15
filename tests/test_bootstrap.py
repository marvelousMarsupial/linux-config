"""Desktop setup boundaries using local files and mocked privileged/network actions."""
import hashlib
import importlib.machinery
import importlib.util
import io
import json
import os
import shutil
import subprocess
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
        self.assertIn('enable --now nftables.service', result.stdout)
        self.assertIn('reload nftables.service', result.stdout)
        self.assertIn('nft list table inet linux_config', result.stdout)
        self.assertFalse((self.home / '.local').exists())
        self.assertNotIn('unexpected-sudo', result.stderr)

    def test_firewall_backs_up_both_files_and_reloads_without_global_flush(self):
        policy = '/etc/nftables.conf'
        override = '/etc/systemd/system/nftables.service.d/10-linux-config.conf'
        files = {policy: b'previous firewall', override: b'previous override'}
        commands = []

        def command(*args, **kwargs):
            args = tuple(map(str, args))
            commands.append(args)
            if args[1:3] == ('cp', '-a'):
                files[args[-1]] = files[args[-2]]
            elif args[1:3] == ('install', '-Dm644'):
                files[args[-1]] = Path(args[-2]).read_bytes()

        def exists(args):
            self.assertEqual(args[:3], ['sudo', 'test', '-e'])
            return subprocess.CompletedProcess(args, 0 if args[-1] in files else 1)

        with patch.object(setup, 'run', side_effect=command), \
             patch.object(setup.subprocess, 'run', side_effect=exists):
            setup.firewall(False)
        self.assertEqual(files[policy], (REPO / 'system/nftables.conf').read_bytes())
        self.assertEqual(files[override], (REPO / 'system/systemd/nftables.service.d/10-linux-config.conf').read_bytes())
        backups = {path: data for path, data in files.items() if path.startswith('/var/backups/')}
        self.assertEqual(set(backups.values()), {b'previous firewall', b'previous override'})
        self.assertEqual(commands[0][:4], ('sudo', 'nft', '--check', '--file'))
        self.assertEqual(commands[-6:], [
            ('sudo', 'systemctl', 'daemon-reload'),
            ('sudo', 'systemctl', 'enable', '--now', 'nftables.service'),
            ('sudo', 'systemctl', 'reload', 'nftables.service'),
            ('sudo', 'systemctl', 'is-enabled', '--quiet', 'nftables.service'),
            ('sudo', 'systemctl', 'is-active', '--quiet', 'nftables.service'),
            ('sudo', 'nft', 'list', 'table', 'inet', 'linux_config'),
        ])
        self.assertFalse(any('restart' in cmd or 'flush' in cmd for cmd in commands))

    def test_firewall_validation_failure_prevents_installation(self):
        with patch.object(setup, 'run', side_effect=subprocess.CalledProcessError(1, 'nft')) as command, \
             patch.object(setup.subprocess, 'run') as probe:
            with self.assertRaises(subprocess.CalledProcessError):
                setup.firewall(False)
        self.assertEqual(command.call_count, 1)
        probe.assert_not_called()

    def test_fresh_firewall_failure_propagates_without_claiming_verification(self):
        for failure in ('reload', 'is-enabled', 'is-active', 'list'):
            with self.subTest(failure=failure):
                commands = []
                def command(*args, **kwargs):
                    commands.append(tuple(map(str, args)))
                    if failure in args:
                        raise subprocess.CalledProcessError(1, args)
                with patch.object(setup, 'run', side_effect=command), \
                     patch.object(setup.subprocess, 'run', return_value=subprocess.CompletedProcess([], 1)), \
                     patch.object(Path, 'is_symlink', return_value=False):
                    with self.assertRaises(subprocess.CalledProcessError):
                        setup.firewall(False)
                self.assertIn(failure, commands[-1])
                self.assertFalse(any('cp' in cmd for cmd in commands))

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

    def test_each_architecture_requests_its_own_release_assets(self):
        for machine, yazi, zen in (('x86_64', 'yazi-x86_64-unknown-linux-musl.zip', 'zen.linux-x86_64.tar.xz'),
                                   ('aarch64', 'yazi-aarch64-unknown-linux-musl.zip', 'zen.linux-aarch64.tar.xz')):
            with self.subTest(machine=machine), patch.object(setup.shutil, 'which', return_value=None), \
                 patch.object(setup.subprocess, 'run') as command, \
                 patch.object(setup, 'release_asset', side_effect=OSError('offline')) as download:
                command.return_value.stdout = ''
                with self.assertRaises(OSError):
                    setup.applications(self.home, False, machine=machine)
                self.assertEqual(download.call_args.args[:2], ('sxyazi/yazi', yazi))
            with self.assertRaises(KeyError):
                setup.applications(self.home, True, machine='armv7l')
        self.assertEqual(setup.ARCHITECTURES['aarch64']['zen'], 'zen.linux-aarch64.tar.xz')

    def test_vm_packages_are_x86_only_and_host_settings_follow_them(self):
        common = set(setup.package_list(REPO / 'packages.txt'))
        x86 = set(setup.package_list(REPO / 'packages.x86_64.txt'))
        self.assertTrue({'ovmf', 'qemu-system-x86', 'libvirt-daemon-system', 'virt-manager'} <= x86)
        self.assertFalse(common & x86)
        self.assertFalse(any('virt' in name or 'qemu' in name for name in common))
        self.assertEqual(setup.package_list(REPO / 'packages.aarch64.txt'), [])
        for virtualization in (True, False):
            with self.subTest(virtualization=virtualization), patch.object(setup, 'run') as run, \
                 patch.object(setup.subprocess, 'run') as command:
                command.return_value.stdout = 'en_us.utf8'
                command.return_value.returncode = 0
                setup.host_settings('someone', True, virtualization)
                calls = [call.args for call in run.call_args_list]
                self.assertIn(('sudo', 'usermod', '-aG', 'video,libvirt,kvm' if virtualization else 'video', 'someone'), calls)
                enabled = next(call for call in calls if call[:3] == ('sudo', 'systemctl', 'enable'))
                self.assertEqual('libvirtd.service' in enabled, virtualization)

    def test_other_username_is_rendered_and_repeat_install_is_clean(self):
        result = run(str(REPO / 'scripts/install'), '--home', str(self.home))
        self.assertEqual(result.returncode, 0, result.stderr)
        for rel in ('.config/theme/config.toml', '.local/share/applications/zen.desktop'):
            content = (self.home / rel).read_text()
            self.assertNotIn('/home/user/', content)
            self.assertIn(str(self.home), content)
        # The Sway config needs no rendering: its exec commands expand a tilde.
        sway = (self.home / '.config/sway/config').read_text()
        self.assertNotIn('/home/', sway)
        self.assertIn('set $bin ~/.local/bin', sway)
        result = run(str(REPO / 'scripts/install'), '--home', str(self.home))
        self.assertIn('0 file(s) changed.', result.stdout)
