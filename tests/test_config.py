"""Offline integration checks; all installation/theme mutations use temporary homes."""
import importlib.machinery
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib
import unittest
import sys

sys.dont_write_bytecode = True

REPO = Path(__file__).resolve().parent.parent


def run(*args, env=None, **kwargs):
    return subprocess.run(args, env=env, capture_output=True, text=True, timeout=30, **kwargs)


def load_script(name):
    loader = importlib.machinery.SourceFileLoader(name, str(REPO / 'dotfiles/.local/bin' / name))
    spec = importlib.util.spec_from_loader(name, loader)
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    return module


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
                tomllib.loads((self.home / '.config/yazi/theme.toml').read_text())
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


@unittest.skipUnless(shutil.which('batcat'), 'batcat required')
class BatTests(unittest.TestCase):
    def setUp(self):
        self.module = load_script('bat')
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)

    def frame(self, names, width=79, style='numbers,grid,header'):
        env = dict(os.environ, BAT_CONFIG_PATH='/dev/null')
        result = run('batcat', '--paging=never', '--color=always', '--decorations=always',
                     f'--terminal-width={width}', f'--style={style}', *map(str, names), env=env)
        self.assertEqual(result.returncode, 0, result.stderr)
        return list(self.module.fitted_frames(result.stdout.splitlines(keepends=True)))

    def test_fitted_frame_colors_and_unicode(self):
        path = self.home / 'short.txt'
        path.write_text('short\nlonger text\n界e\u0301\n')
        lines = self.frame([path])
        widths = [self.module.display_width(line.rstrip('\n')) for line in lines]
        self.assertEqual(len(set(widths)), 1, widths)
        self.assertLess(widths[0], 80)
        plain = [self.module.ANSI.sub('', line).rstrip('\n') for line in lines]
        self.assertTrue(plain[0].endswith('╮'))
        self.assertTrue(plain[-1].endswith('╯'))
        self.assertTrue(all(line.endswith('│') for line in plain[1:-1] if '├' not in line))
        self.assertIn('\x1b[', ''.join(lines))
        self.assertEqual(self.module.display_width('界e\u0301'), 3)

    def test_long_file_stays_inside_terminal(self):
        path = self.home / 'long.txt'
        path.write_text('x' * 300 + '\n')
        lines = self.frame([path], width=39)
        self.assertTrue(all(self.module.display_width(line.rstrip('\n')) == 40 for line in lines))

    def test_multiple_files_and_empty_file(self):
        paths = [self.home / name for name in ['a.txt', 'longer-filename.txt', 'empty.txt']]
        paths[0].write_text('a\n')
        paths[1].write_text('a bit longer\n')
        paths[2].write_text('')
        plain = ''.join(self.module.ANSI.sub('', line) for line in self.frame(paths))
        self.assertEqual(plain.count('╭'), 3)
        self.assertEqual(plain.count('╯'), 3)
        self.assertIn('a bit longer', plain)

    def test_piped_output_and_error_exit(self):
        path = self.home / 'raw.txt'
        path.write_text('keep\ttabs\nand content\n')
        result = run(str(REPO / 'dotfiles/.local/bin/bat'), str(path))
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, path.read_text())
        result = run(str(REPO / 'dotfiles/.local/bin/bat'), str(self.home / 'absent'))
        self.assertNotEqual(result.returncode, 0)

    def test_plain_input_and_large_spooled_file(self):
        self.assertEqual(list(self.module.fitted_frames(['plain\n'])), ['plain\n'])
        # Enough content to exceed the one-MiB memory buffer.
        lines = ['─────┬' + '─' * 70 + '\n', '     │ heading\n', '─────┼' + '─' * 70 + '\n']
        lines.extend(['   1 │ ' + 'x' * 60 + '\n'] * 18000)
        lines.append('─────┴' + '─' * 70 + '\n')
        result = list(self.module.fitted_frames(lines))
        self.assertEqual(len(result), len(lines))
        self.assertTrue(self.module.ANSI.sub('', result[-1]).rstrip().endswith('╯'))


class ProjectTests(unittest.TestCase):
    def test_minimal_packages_and_retired_integrations(self):
        packages = {line for line in (REPO / 'packages.txt').read_text().splitlines()
                    if line and not line.startswith('#')}
        self.assertTrue({'ipython3', 'jq', 'bat', 'libvirt-clients', 'virt-manager'} <= packages)
        self.assertFalse(packages & {'podman', 'docker.io', 'incus', 'lxc', 'build-essential',
                                     'clangd', 'clang-format', 'nodejs', 'npm', 'tmux', 'wireguard-tools'})
        self.assertFalse((REPO / 'dotfiles/.config/nvim/lua/plugins/obsidian.lua').exists())
        self.assertFalse((REPO / 'dotfiles/.local/bin/vm-menu').exists())
        self.assertFalse((REPO / 'dotfiles/.local/bin/dictate').exists())
        self.assertFalse((REPO / 'dotfiles/.local/bin/whisper-server').exists())
        self.assertNotIn('get_stt', (REPO / 'dotfiles/.local/bin/status-panel').read_text())
        self.assertNotIn('SIGUSR1', (REPO / 'dotfiles/.config/theme/config.toml').read_text())

    def test_setup_cli_rejects_unknown_options(self):
        result = run(str(REPO / 'scripts/setup-editor'), '--help')
        self.assertEqual(result.returncode, 0)
        self.assertIn('--dev-tools', result.stdout)
        self.assertNotEqual(run(str(REPO / 'scripts/setup-editor'), '--nonsense').returncode, 0)



class DesktopHelperTests(unittest.TestCase):
    def test_wifi_scan_escaped_ssids_duplicates_and_rescan_timeout(self):
        from unittest.mock import patch
        wifi = load_script('wifi-menu')
        sample = 'Cafe\\:Lab:40:WPA2\nGuest:85:--\nCafe\\:Lab:90:WPA3\n:50:WPA2\nOdd::--\n'
        with patch.object(wifi.subprocess, 'run', side_effect=subprocess.TimeoutExpired('nmcli', 5)), \
             patch.object(wifi, 'run_output', return_value=sample):
            networks = wifi.scan_networks()
        self.assertEqual(networks, [('Cafe:Lab', 90, True), ('Guest', 85, False), ('Odd', 0, False)])
        self.assertTrue(wifi.is_auth_error('Secrets were required'))
        self.assertFalse(wifi.is_auth_error('No Wi-Fi device found'))
        self.assertEqual(wifi.split_nmcli(r'a\\b\:c:80:WPA2'), ['a\\b:c', '80', 'WPA2'])

    def test_wifi_cancel_does_not_connect_or_delete_profiles(self):
        from unittest.mock import patch
        wifi = load_script('wifi-menu')
        with patch.object(wifi, 'saved_connections', return_value=set()), \
             patch.object(wifi, 'prompt_password', return_value=None), \
             patch.object(wifi, 'nmcli_connect') as connect:
            wifi.connect('Private', True)
        connect.assert_not_called()


    def test_battery_missing_and_malformed_readings(self):
        battery = load_script('battery-monitor')
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self.assertIsNone(battery.get_battery_level(root))
            self.assertEqual(battery.get_battery_status(root), '')
            (root / 'capacity').write_text('not a number')
            self.assertIsNone(battery.get_battery_level(root))
            (root / 'capacity').write_text('42\n')
            (root / 'status').write_text('Charging\n')
            self.assertEqual(battery.get_battery_level(root), 42)
            self.assertEqual(battery.get_battery_status(root), 'Charging')

    def test_status_panel_handles_missing_tools(self):
        from unittest.mock import patch
        panel = load_script('status-panel')
        with patch.object(panel.subprocess, 'run', side_effect=FileNotFoundError):
            self.assertEqual(panel.run(['absent']), '')
        self.assertEqual(panel.trunc('abcdef', 4), 'abc…')
        self.assertEqual(panel.center('abc', 7), '  abc')
        self.assertEqual(panel.center('too long', 2), 'too long')

    def test_screenshot_selection_cancel_full_and_capture_failure(self):
        with tempfile.TemporaryDirectory() as d:
            home = Path(d)
            tools = home / 'bin'
            tools.mkdir()
            log = home / 'actions.log'
            stubs = {
                'slurp': '#!/bin/sh\n[ "$CANCEL_SELECTION" = 1 ] && exit 1\necho "0,0 100x100"\n',
                'grim': '#!/bin/sh\n[ "$FAIL_CAPTURE" = 1 ] && exit 1\nfor out; do :; done\nprintf PNG > "$out"\n',
                'wl-copy': '#!/bin/sh\nprintf "copy\\n" >> "$ACTION_LOG"\ncat > "$HOME/copied.png"\n',
                'notify-send': '#!/bin/sh\nprintf "notify\\n" >> "$ACTION_LOG"\n',
            }
            for name, content in stubs.items():
                path = tools / name
                path.write_text(content)
                path.chmod(0o755)
            env = dict(os.environ, HOME=str(home), PATH=f'{tools}:/usr/bin:/bin',
                       ACTION_LOG=str(log), CANCEL_SELECTION='1', FAIL_CAPTURE='0')
            result = run(str(REPO / 'dotfiles/.local/bin/screenshot'), 'select', env=env)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(list((home / 'Pictures').glob('*.png')))
            self.assertFalse(log.exists())
            env['CANCEL_SELECTION'] = '0'
            result = run(str(REPO / 'dotfiles/.local/bin/screenshot'), 'full', env=env)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((home / 'copied.png').read_bytes(), b'PNG')
            self.assertEqual(log.read_text(), 'copy\nnotify\n')
            log.unlink()
            env['FAIL_CAPTURE'] = '1'
            result = run(str(REPO / 'dotfiles/.local/bin/screenshot'), 'full', env=env)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(log.exists())


@unittest.skipUnless(shutil.which('nvim') and (Path.home() / '.local/share/nvim/lazy/lazy.nvim').exists(),
                     'Neovim and installed plugins required')
class NeovimTests(unittest.TestCase):
    def test_filetypes_prose_saves_plugins_and_language_configs(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            config = root / 'config/nvim'
            shutil.copytree(REPO / 'dotfiles/.config/nvim', config)
            env = dict(os.environ, XDG_CONFIG_HOME=str(root / 'config'),
                       XDG_DATA_HOME=str(Path.home() / '.local/share'),
                       XDG_CACHE_HOME=str(root / 'cache'), XDG_STATE_HOME=str(root / 'state'),
                       LINUX_CONFIG_TEST_DIR=d)
            result = run('nvim', '--headless', '-c',
                         f'lua dofile({json.dumps(str(REPO / "tests/nvim_smoke.lua"))})', '+qa', env=env)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertNotIn('Error', result.stderr)
            self.assertIn('eight filetypes', result.stderr)


class SetupTests(unittest.TestCase):
    def test_default_and_optional_install_branches(self):
        for development in (False, True):
            with self.subTest(development=development), tempfile.TemporaryDirectory() as d:
                home = Path(d)
                tools = home / '.local/bin'
                tools.mkdir(parents=True)
                log = home / 'commands.log'
                for name in ('sudo', 'nvim'):
                    stub = tools / name
                    stub.write_text('#!/bin/sh\nprintf "%s\\n" "' + name + ' $*" >> "$SETUP_LOG"\n')
                    stub.chmod(0o755)
                env = dict(os.environ, HOME=d, PATH=f'{tools}:/usr/bin:/bin', SETUP_LOG=str(log))
                args = ['--dev-tools'] if development else []
                result = run(str(REPO / 'scripts/setup-editor'), *args, env=env)
                self.assertEqual(result.returncode, 0, result.stderr)
                commands = log.read_text()
                self.assertIn('ipython3 jq bat', commands)
                self.assertEqual('apt-get install -y build-essential clangd clang-format nodejs npm' in commands, development)
                self.assertEqual('dofile(vim.env.LINUX_CONFIG_EDITOR_SETUP)' in commands, development)
                self.assertTrue((home / '.config/nvim/init.lua').exists())


class ScriptSafetyTests(unittest.TestCase):
    def test_wifi_preserves_saved_profiles_and_keeps_password_out_of_argv(self):
        from unittest.mock import patch
        wifi = load_script('wifi-menu')
        with patch.object(wifi, 'saved_connections', return_value={'Private'}), \
             patch.object(wifi, 'nmcli_connect', return_value=(False, 'Secrets were required')) as connect, \
             patch.object(wifi, 'prompt_password', return_value=None), \
             patch.object(wifi.subprocess, 'run') as commands:
            wifi.connect('Private', True)
        connect.assert_called_once_with('Private')
        commands.assert_not_called()
        with patch.object(wifi.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, '', '')) as command:
            self.assertEqual(wifi.nmcli_connect('Private', 'secret-value'), (True, ''))
        self.assertNotIn('secret-value', command.call_args.args[0])
        self.assertEqual(command.call_args.kwargs['input'], 'secret-value\n')

    def test_installer_refuses_parent_symlink_outside_home(self):
        with tempfile.TemporaryDirectory() as d, tempfile.TemporaryDirectory() as outside:
            home = Path(d)
            (home / '.config').symlink_to(outside, target_is_directory=True)
            result = run(str(REPO / 'scripts/install'), '--home', d)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('outside destination home', result.stderr)
            self.assertFalse(list(Path(outside).iterdir()))

    def test_note_names_are_unique_private_and_menu_paths_are_checked(self):
        with tempfile.TemporaryDirectory(prefix='notes & home ') as d:
            home = Path(d)
            tools = home / 'bin'
            tools.mkdir()
            log = home / 'actions'
            for name, body in {
                'swaymsg': 'echo "[]"',
                'foot': 'printf "%s\\n" "$@" >> "$ACTION_LOG"',
                'fuzzel': 'cat >/dev/null; printf "%s\\n" "$PICK"; exit "${PICK_EXIT:-0}"',
            }.items():
                p = tools / name
                p.write_text('#!/bin/sh\n' + body + '\n')
                p.chmod(0o755)
            env = dict(os.environ, HOME=d, PATH=f'{tools}:/usr/bin:/bin', ACTION_LOG=str(log))
            for _ in range(2):
                result = run(str(REPO / 'dotfiles/.local/bin/notes-toggle'), env=env)
                self.assertEqual(result.returncode, 0, result.stderr)
            notes = list((home / 'vault').glob('*.md'))
            self.assertEqual(len(notes), 2)
            self.assertTrue(all(n.read_text() == '# \n' and n.stat().st_mode & 0o777 == 0o600 for n in notes))
            log.unlink()
            (home / 'outside.md').write_text('private')
            for pick in ('../outside.md', '-missing.md'):
                result = run(str(REPO / 'dotfiles/.local/bin/notes-picker'), env=dict(env, PICK=pick))
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(log.exists())
            result = run(str(REPO / 'dotfiles/.local/bin/notes-picker'), env=dict(env, PICK=notes[0].name))
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn(str(notes[0]), log.read_text())

    def test_cancelled_power_and_password_menus_do_nothing(self):
        with tempfile.TemporaryDirectory() as d:
            home = Path(d)
            tools = home / 'bin'
            tools.mkdir()
            (home / '.password-store').mkdir()
            (home / '.password-store/example.gpg').write_text('encrypted')
            for name in ('fuzzel', 'systemctl', 'swaymsg', 'swaylock', 'foot'):
                p = tools / name
                p.write_text('#!/bin/sh\n' + ('cat >/dev/null; echo Reboot; exit 1\n' if name == 'fuzzel' else 'echo action >> "$HOME/actions"\n'))
                p.chmod(0o755)
            env = dict(os.environ, HOME=d, PATH=f'{tools}:/usr/bin:/bin')
            for script in ('power-menu', 'pass-picker'):
                result = run(str(REPO / 'dotfiles/.local/bin' / script), env=env)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertFalse((home / 'actions').exists())

    def test_battery_rejects_invalid_thresholds_and_levels(self):
        result = run(str(REPO / 'dotfiles/.local/bin/battery-monitor'), '--interval', '0')
        self.assertNotEqual(result.returncode, 0)
        battery = load_script('battery-monitor')
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / 'capacity').write_text('110')
            self.assertIsNone(battery.get_battery_level(Path(d)))


if __name__ == '__main__':
    unittest.main(verbosity=2)
