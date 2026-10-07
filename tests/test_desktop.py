"""Desktop regression tests."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from support import REPO, load_script, run


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
                'swaymsg': 'echo "{}"',
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
