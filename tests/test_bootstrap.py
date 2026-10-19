"""Desktop setup boundaries using local files and mocked privileged/network actions."""
import hashlib
import importlib.machinery
import tomllib
import sys
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

    def test_autologin_renders_the_user_backs_up_and_is_idempotent(self):
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
        destination = local_path('/etc/systemd/system/getty@tty1.service.d/autologin.conf')
        destination.parent.mkdir(parents=True)
        destination.write_text('old drop-in\n')
        with patch.object(setup, 'Path', side_effect=local_path), patch.object(setup, 'run', side_effect=command):
            setup.autologin('someone', False)
            text = destination.read_text()
            self.assertIn('ExecStart=-/sbin/agetty --autologin someone --noclear %I $TERM\n', text)
            self.assertIn('ExecStart=\n', text)
            self.assertEqual(commands[-1], ('sudo', 'systemctl', 'daemon-reload'))
            self.assertFalse(any('restart' in c or 'getty' in ' '.join(c) and c[1] == 'systemctl' for c in commands))
            backups = list((self.home / 'var/backups').rglob('autologin.conf'))
            self.assertEqual([b.read_text() for b in backups], ['old drop-in\n'])
            commands.clear()
            setup.autologin('someone', False)
            self.assertEqual(commands, [])
            setup.autologin('other', True)
            # A dry run only announces the reload through run(dry=True); nothing is installed.
            self.assertEqual(commands, [('sudo', 'systemctl', 'daemon-reload')])
            self.assertEqual(destination.read_text(), text)

    def test_ifupdown_handover_keeps_static_stanzas_and_migrates_wifi(self):
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
        interfaces = local_path('/etc/network/interfaces')
        interfaces.parent.mkdir(parents=True)
        original = ('source /etc/network/interfaces.d/*\n\n'
                    'auto lo\niface lo inet loopback\n\n'
                    'allow-hotplug enp0s3\niface enp0s3 inet dhcp\n\n'
                    'allow-hotplug wlp2s0\niface wlp2s0 inet dhcp\n\twpa-ssid "Home Net"\n\twpa-psk s3cret pass\n\n'
                    'auto eth1\niface eth1 inet static\n\taddress 10.0.0.5/24\n\tgateway 10.0.0.1\n')
        interfaces.write_text(original)
        with patch.object(setup, 'Path', side_effect=local_path), patch.object(setup, 'run', side_effect=command), \
                patch('sys.stdout', new_callable=io.StringIO) as out:
            setup.network_manager(True)
            self.assertEqual(commands, [])
            self.assertEqual(interfaces.read_text(), original)
            self.assertIn('eth1 has a static', out.getvalue())
            self.assertIn('Hand enp0s3, wlp2s0 from ifupdown', out.getvalue())
            self.assertIn('connection "Home Net" for wlp2s0', out.getvalue())
            setup.network_manager(False)
        text = interfaces.read_text()
        for kept in ('source /etc/network/interfaces.d/*', 'auto lo\n', 'iface lo inet loopback',
                     'auto eth1\n', 'iface eth1 inet static', '\taddress 10.0.0.5/24', '\tgateway 10.0.0.1'):
            self.assertIn(kept, text)
        for commented in ('allow-hotplug enp0s3', 'iface enp0s3 inet dhcp', 'allow-hotplug wlp2s0',
                          'iface wlp2s0 inet dhcp', '\twpa-ssid "Home Net"', '\twpa-psk s3cret pass'):
            self.assertIn('# ' + commented, text)
            self.assertNotIn('\n' + commented, text)
        keyfile = local_path('/etc/NetworkManager/system-connections/Home Net.nmconnection')
        connection = keyfile.read_text()
        for line in ('id=Home Net', 'interface-name=wlp2s0', 'ssid=Home Net', 'key-mgmt=wpa-psk', 'psk=s3cret pass', 'method=auto'):
            self.assertIn(line + '\n', connection)
        self.assertRegex(connection, r'uuid=[0-9a-f-]{36}\n')
        installs = [c for c in commands if c[1] == 'install' and c[2] == '-Dm600']
        self.assertEqual([c[-1] for c in installs], [str(interfaces), str(keyfile)])
        self.assertEqual(commands[-1], ('sudo', 'nmcli', 'connection', 'reload'))
        self.assertFalse(any('s3cret' in ' '.join(c) for c in commands))
        backups = list((self.home / 'var/backups').rglob('interfaces'))
        self.assertEqual([b.read_text() for b in backups], [original])
        # A rerun finds only commented stanzas and does nothing.
        commands.clear()
        with patch.object(setup, 'Path', side_effect=local_path), patch.object(setup, 'run', side_effect=command):
            setup.network_manager(False)
            self.assertEqual(commands, [])
            interfaces.write_text('auto lo\niface lo inet loopback\n')
            setup.network_manager(False)
            self.assertEqual(commands, [])

    def test_ifupdown_handover_reads_root_only_file_with_sudo(self):
        commands = []
        def local_path(value):
            value = str(value)
            return self.home / value.lstrip('/') if value.startswith(('/etc/', '/var/')) else Path(value)
        def command(*args, **kwargs):
            commands.append(tuple(map(str, args)))
            if args[1] == 'cat':
                return subprocess.CompletedProcess(args, 0, 'allow-hotplug wlp4s0\niface wlp4s0 inet dhcp\n')
        interfaces = local_path('/etc/network/interfaces')
        interfaces.parent.mkdir(parents=True)
        interfaces.write_text('')
        real_read = Path.read_text
        def read_text(path, *args, **kwargs):
            if path == interfaces:
                raise PermissionError(13, 'Permission denied')
            return real_read(path, *args, **kwargs)
        with patch.object(setup, 'Path', side_effect=local_path), patch.object(setup, 'run', side_effect=command), \
                patch.object(Path, 'read_text', read_text), patch('sys.stdout', new_callable=io.StringIO) as out:
            setup.network_manager(True)
            self.assertEqual(commands, [])
            self.assertIn('readable only by root', out.getvalue())
            setup.network_manager(False)
        self.assertEqual(commands[0], ('sudo', 'cat', str(interfaces)))
        self.assertIn('Hand wlp4s0 from ifupdown', out.getvalue())

    def test_dry_run_does_not_write_home_or_call_sudo(self):
        binary = self.home / 'bin'
        binary.mkdir()
        sudo = binary / 'sudo'
        sudo.write_text('#!/bin/sh\necho unexpected-sudo >&2\nexit 99\n')
        sudo.chmod(0o755)
        result = run(str(REPO / 'scripts/setup'), '--dry-run', '--autologin',
                     env=dict(os.environ, HOME=str(self.home), PATH=f'{binary}:/usr/bin:/bin'))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('Plan complete.', result.stdout)
        # The dry run reads the real /etc; a host that already has the drop-in says so instead.
        self.assertTrue('then configure autologin for' in result.stdout
                        or 'on tty1 already configured' in result.stdout, result.stdout)
        browser = setup.pinned_releases()[setup.BROWSER_PACKAGE]
        if setup.installed_version(setup.BROWSER_PACKAGE) != browser['tag'].removeprefix('v'):
            # dpkg's state is real even in a dry run; a host with the pin installed plans no download.
            self.assertIn(f'marvelousMarsupial/firefoxconfig {browser["tag"]}: '
                          f'firefoxconfig_{browser["tag"].removeprefix("v")}_all.deb', result.stdout)
        self.assertIn('--skip-apt', result.stdout)
        self.assertTrue('configure OpenSnitch UI socket:' in result.stdout
                        or 'enable --now opensnitch.service' in result.stdout, result.stdout)
        self.assertIn('enable --now opensnitch.service', result.stdout)
        self.assertIn('enable --now nftables.service', result.stdout)
        self.assertIn('reload nftables.service', result.stdout)
        self.assertIn('nft list table inet linux_config', result.stdout)
        self.assertFalse((self.home / '.local').exists())
        self.assertNotIn('unexpected-sudo', result.stderr)

    def test_dry_run_re_renders_the_selected_palette(self):
        env = dict(os.environ, HOME=str(self.home), PATH='/usr/bin:/bin')
        fresh = run(str(REPO / 'scripts/setup'), '--dry-run', env=env)
        self.assertEqual(fresh.returncode, 0, fresh.stderr)
        self.assertIn('theme apply summer-dusk', fresh.stdout)
        current = self.home / '.cache/theme/current'
        current.parent.mkdir(parents=True)
        current.write_text('kanagawa\n')
        rerun = run(str(REPO / 'scripts/setup'), '--dry-run', env=env)
        self.assertEqual(rerun.returncode, 0, rerun.stderr)
        self.assertIn('theme apply kanagawa', rerun.stdout)
        self.assertNotIn('theme apply summer-dusk', rerun.stdout)
        self.assertNotIn('autologin', rerun.stdout)

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

    def test_pinned_download_and_rejected_digest(self):
        payload = b'release fixture'
        urls = []

        def response(url):
            urls.append(url)
            return io.BytesIO(payload)
        destination = self.home / 'download'
        with patch.object(setup, 'request', side_effect=response):
            setup.release_asset('example/app', 'fixture.zip', destination, 'v1', hashlib.sha256(payload).hexdigest())
            self.assertEqual(destination.read_bytes(), payload)
            self.assertEqual(urls, ['https://github.com/example/app/releases/download/v1/fixture.zip'])
            with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
                setup.release_asset('example/app', 'fixture.zip', destination, 'v1', '0' * 64)
            # A hosted package: {version} expands and the entry's url replaces GitHub.
            setup.release_asset('example/app', 'app_{version}_all.deb', destination, 'v1.2',
                                hashlib.sha256(payload).hexdigest(), 'https://packages.example/pool')
            self.assertEqual(urls[-1], 'https://packages.example/pool/app_1.2_all.deb')

    def test_pins_cover_every_architecture_and_reject_bad_entries(self):
        releases = setup.pinned_releases()
        for assets in setup.ARCHITECTURES.values():
            self.assertIn(assets['yazi'], releases['yazi']['assets'])
        self.assertIn('CascadiaCode.zip', releases['font']['assets'])
        browser = releases[setup.BROWSER_PACKAGE]
        self.assertEqual(browser['repository'], 'marvelousMarsupial/firefoxconfig')
        self.assertEqual(list(browser['assets']), ['firefoxconfig_{version}_all.deb'])
        self.assertEqual(setup.asset_name('firefoxconfig_{version}_all.deb', 'v0.1.0'), 'firefoxconfig_0.1.0_all.deb')
        bad = self.home / 'releases.toml'
        for text, message in (('[yazi]\nrepository = "x"\ntag = "v1"\n', 'invalid repository'),
                              ('[yazi]\nrepository = "a/b"\ntag = "../x"\n', 'invalid tag'),
                              ('[yazi]\nrepository = "a/b"\ntag = "v1"\nurl = "http://x/"\n', 'HTTPS'),
                              ('[yazi]\nrepository = "a/b"\ntag = "v1"\n[yazi.assets]\n"a.zip" = "short"\n',
                               'SHA-256')):
            bad.write_text(text)
            with self.assertRaisesRegex(ValueError, message):
                setup.pinned_releases(bad)

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
        pinned = setup.pinned_releases()[setup.BROWSER_PACKAGE]['tag'].removeprefix('v')
        with patch.object(setup.shutil, 'which', return_value='/bin/existing'), \
             patch.object(setup.subprocess, 'run') as command, \
             patch.object(setup, 'installed_version', return_value=pinned), \
             patch.object(setup, 'release_asset') as download:
            command.return_value.stdout = 'CaskaydiaCove Nerd Font Mono'
            setup.applications(self.home, False)
            download.assert_not_called()

    def test_installed_version_reads_dpkg_and_tolerates_its_absence(self):
        query = subprocess.CompletedProcess([], 0, stdout='0.1.0\n', stderr='')
        with patch.object(setup.subprocess, 'run', return_value=query) as command:
            self.assertEqual(setup.installed_version('firefoxconfig'), '0.1.0')
            self.assertEqual(command.call_args.args[0][:2], ['dpkg-query', '-W'])
        with patch.object(setup.subprocess, 'run', return_value=subprocess.CompletedProcess([], 1, stdout='', stderr='')):
            self.assertIsNone(setup.installed_version('firefoxconfig'))
        with patch.object(setup.subprocess, 'run', side_effect=FileNotFoundError):
            self.assertIsNone(setup.installed_version('firefoxconfig'))

    def test_failed_replacement_restores_previous_installation(self):
        dest = self.home / 'installed'
        dest.write_text('previous')
        backup = self.home / 'backup'
        with self.assertRaises(OSError):
            setup.replace(self.home / 'missing', dest, backup)
        self.assertEqual(dest.read_text(), 'previous')
        self.assertFalse(backup.exists())

    def test_fresh_home_installs_all_missing_applications(self):
        pinned = setup.pinned_releases()[setup.BROWSER_PACKAGE]['tag'].removeprefix('v')
        cache = self.home / '.cache/mozilla/firefox/abc.default-esr/startupCache'
        cache.mkdir(parents=True)
        (cache / 'scriptCache.bin').write_bytes(b'stale')

        def download(repository, name, destination, tag, digest, url=None):
            self.assertRegex(digest, r'^[0-9a-f]{64}$')
            if repository == 'marvelousMarsupial/firefoxconfig':
                self.assertEqual(name, 'firefoxconfig_{version}_all.deb')
                self.assertIsNone(url)
                destination.write_bytes(b'!<arch>\ndeb fixture')
            else:
                with zipfile.ZipFile(destination, 'w') as archive:
                    if repository == 'sxyazi/yazi':
                        archive.writestr('yazi-release/yazi', '#!/bin/sh\nexit 0\n')
                        archive.writestr('yazi-release/ya', '#!/bin/sh\nexit 0\n')
                    else:
                        archive.writestr('CaskaydiaCoveNerdFont-Regular.ttf', 'fixture')

        with patch.object(setup.shutil, 'which', return_value=None), \
             patch.object(setup, 'installed_version', side_effect=[None, pinned]), \
             patch.object(setup, 'firefox_running', return_value=False), \
             patch.object(setup, 'release_asset', side_effect=download) as fetch, \
             patch.object(setup, 'run') as command:
            setup.applications(self.home, False)
            self.assertEqual(fetch.call_count, 3)
        installs = [call.args for call in command.call_args_list if call.args[:2] == ('sudo', 'dpkg')]
        self.assertEqual(len(installs), 1)
        self.assertEqual(installs[0][2], '-i')
        self.assertEqual(Path(installs[0][3]).name, f'firefoxconfig_{pinned}_all.deb')
        self.assertTrue(Path(installs[0][3]).is_relative_to(self.home / '.local'))
        self.assertFalse(cache.exists())
        for executable in ('.local/bin/yazi', '.local/bin/ya'):
            path = self.home / executable
            self.assertTrue(path.is_file())
            self.assertEqual(path.stat().st_mode & 0o777, 0o755)
        self.assertTrue(list((self.home / '.local/share/fonts').rglob('*.ttf')))
        self.assertEqual(list((self.home / '.local').glob('linux-config-*')), [])

    def test_browser_package_that_does_not_install_stops_setup(self):
        def download(repository, name, destination, tag, digest, url=None):
            destination.write_bytes(b'!<arch>\ndeb fixture')
        with patch.object(setup.shutil, 'which', return_value='/bin/existing'), \
             patch.object(setup.subprocess, 'run') as command, \
             patch.object(setup, 'installed_version', return_value=None), \
             patch.object(setup, 'release_asset', side_effect=download), \
             patch.object(setup, 'run'):
            command.return_value.stdout = 'CaskaydiaCove Nerd Font Mono'
            with self.assertRaisesRegex(ValueError, 'did not install'):
                setup.applications(self.home, False)
        self.assertEqual(list((self.home / '.local').glob('linux-config-*')), [])

    def test_each_architecture_requests_its_own_release_assets(self):
        for machine, yazi in (('x86_64', 'yazi-x86_64-unknown-linux-musl.zip'),
                              ('aarch64', 'yazi-aarch64-unknown-linux-musl.zip')):
            with self.subTest(machine=machine), patch.object(setup.shutil, 'which', return_value=None), \
                 patch.object(setup.subprocess, 'run') as command, \
                 patch.object(setup, 'installed_version', return_value=None), \
                 patch.object(setup, 'release_asset', side_effect=OSError('offline')) as download:
                command.return_value.stdout = ''
                with self.assertRaises(OSError):
                    setup.applications(self.home, False, machine=machine)
                self.assertEqual(download.call_args.args[:2], ('sxyazi/yazi', yazi))
            with self.assertRaises(KeyError):
                setup.applications(self.home, True, machine='armv7l')
        # The browser package is architecture independent, so it is not an architecture asset.
        self.assertNotIn(setup.BROWSER_PACKAGE, setup.ARCHITECTURES['aarch64'])

    def test_release_helper_checks_and_bumps_pins(self):
        helper = importlib.machinery.SourceFileLoader('releases', str(REPO / 'scripts/releases')).load_module()
        pins = self.home / 'releases.toml'
        pins.write_text((REPO / 'releases.toml').read_text())
        current = tomllib.loads(pins.read_text())
        newer = {name: ('v99', {asset: 'f' * 64 for asset in release['assets']})
                 for name, release in current.items()}
        newer['font'] = (current['font']['tag'], {})
        newer['firefoxconfig'] = ('v9.9.9', {'firefoxconfig_9.9.9_all.deb': 'e' * 64})
        with patch.object(helper, 'PINS', pins), patch.object(helper, 'latest', side_effect=lambda r: next(
                newer[n] for n, rel in current.items() if rel['repository'] == r)), \
                patch.object(sys, 'argv', ['releases', 'check']):
            self.assertEqual(helper.main(), 1)
            self.assertEqual(pins.read_text(), (REPO / 'releases.toml').read_text())
        with patch.object(helper, 'PINS', pins), patch.object(helper, 'latest', side_effect=lambda r: next(
                newer[n] for n, rel in current.items() if rel['repository'] == r)), \
                patch.object(sys, 'argv', ['releases', 'bump', 'yazi', 'firefoxconfig']):
            self.assertEqual(helper.main(), 0)
        bumped = tomllib.loads(pins.read_text())
        self.assertEqual(bumped['yazi']['tag'], 'v99')
        self.assertEqual(set(bumped['yazi']['assets'].values()), {'f' * 64})
        self.assertEqual(bumped['font'], current['font'])
        # The versioned asset name stays a template; the digest is the new release's.
        self.assertEqual(bumped['firefoxconfig'], {'repository': current['firefoxconfig']['repository'],
                                                    'tag': 'v9.9.9',
                                                    'assets': {'firefoxconfig_{version}_all.deb': 'e' * 64}})
        setup.pinned_releases(pins)
        # A hosted entry keeps its url through a rewrite.
        bumped['firefoxconfig']['url'] = 'https://packages.example/pool'
        pins.write_text(helper.render(bumped))
        self.assertEqual(tomllib.loads(pins.read_text())['firefoxconfig']['url'], 'https://packages.example/pool')
        setup.pinned_releases(pins)

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
                self.assertIn(('sudo', 'usermod', '-aG', 'video,adm,systemd-journal,libvirt,kvm' if virtualization else 'video,adm,systemd-journal', 'someone'), calls)
                enabled = next(call for call in calls if call[:3] == ('sudo', 'systemctl', 'enable'))
                self.assertEqual('libvirtd.service' in enabled, virtualization)

    def test_home_row_mods_install_keyd_only_when_apt_offers_it(self):
        commands = []
        def local_path(value):
            value = str(value)
            return self.home / value.lstrip('/') if value.startswith(('/etc/', '/var/')) else Path(value)
        def command(*args, **kwargs):
            commands.append(tuple(map(str, args)))
            if args[1] == 'install' and '-d' not in args:
                Path(args[-1]).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(args[-2], args[-1])
        dest = local_path('/etc/keyd/laptop.conf')
        for policy in ('', 'keyd:\n  Installed: (none)\n  Candidate: (none)\n'):
            with self.subTest(policy=policy), patch.object(setup, 'run', side_effect=command), \
                    patch.object(setup.subprocess, 'run') as apt, patch('sys.stdout', new_callable=io.StringIO) as out:
                apt.return_value.stdout = policy
                setup.home_row_mods(False)
            self.assertEqual(commands, [])
            self.assertIn('home row modifiers were skipped', out.getvalue())
        def offered(args, **kwargs):
            if args[0] == 'apt-cache':
                return subprocess.CompletedProcess(args, 0, 'keyd:\n  Candidate: 2.5.0-1\n')
            return subprocess.CompletedProcess(args, 1)
        with patch.object(setup, 'Path', side_effect=local_path), patch.object(setup, 'run', side_effect=command), \
                patch.object(setup.subprocess, 'run', side_effect=offered):
            setup.home_row_mods(False)
            self.assertEqual(dest.read_bytes(), (REPO / 'system/keyd/laptop.conf').read_bytes())
            self.assertEqual(commands[0], ('sudo', 'apt-get', 'install', '-y', 'keyd'))
            self.assertEqual(commands[-1], ('sudo', 'keyd', 'reload'))
            # A rerun with the same map leaves the file alone.
            commands.clear()
            setup.home_row_mods(False)
            self.assertFalse(any(c[1] == 'install' and c[2] == '-Dm644' for c in commands))
        config = (REPO / 'system/keyd/laptop.conf').read_text()
        for key, modifier in (('a', 'meta'), ('s', 'alt'), ('d', 'control'), ('f', 'shift'),
                              ('j', 'shift'), ('k', 'control'), ('l', 'alt'), ('semicolon', 'meta')):
            self.assertIn(f'{key} = lettermod({modifier}, {key}, ', config)
        self.assertIn('[ids]\n0001:0001\n', config)

    def test_other_username_is_rendered_and_repeat_install_is_clean(self):
        result = run(str(REPO / 'scripts/install'), '--home', str(self.home))
        self.assertEqual(result.returncode, 0, result.stderr)
        for rel in ('.local/share/applications/firefox-esr.desktop',):
            content = (self.home / rel).read_text()
            self.assertNotIn('/home/user/', content)
            self.assertIn(str(self.home), content)
        self.assertNotIn('/home/', (self.home / '.config/theme/config.toml').read_text())
        # The Sway config needs no rendering: its exec commands expand a tilde.
        sway = (self.home / '.config/sway/config').read_text()
        self.assertNotIn('/home/', sway)
        self.assertIn('set $bin ~/.local/bin', sway)
        result = run(str(REPO / 'scripts/install'), '--home', str(self.home))
        self.assertIn('0 file(s) changed.', result.stdout)
