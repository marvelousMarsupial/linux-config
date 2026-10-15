"""Run user commands with isolated homes and recorded external actions."""
import json
from support import REPO, run
from command_support import CommandSandbox


class CommandTests(CommandSandbox):
    def test_brightness_changes_only_screen_backlights(self):
        self.stub('brightnessctl')
        for direction, step in (('up', '+10%'), ('down', '10%-')):
            with self.subTest(direction=direction):
                self.reset()
                result = self.execute('brightness', direction)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(self.events()[-1]['args'],
                                 ['--class=backlight', '--min-value=1', 'set', step])

    def test_brightness_missing_backlight_never_changes_leds(self):
        self.stub('brightnessctl')
        result = self.execute('brightness', 'up', responses={'brightnessctl': {'code': 1}})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([e['name'] for e in self.events()], ['brightnessctl', 'notify-send'])
        self.assertNotIn('set', self.events()[0]['args'])
        self.assertIn('No screen brightness control found', result.stderr)

    def test_brightness_permission_failure_is_visible(self):
        self.stub('brightnessctl')
        result = self.execute('brightness', 'down', responses={
            'brightnessctl --class=backlight --min-value=1 set 10%-': {'code': 1}})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.events()[-1]['name'], 'notify-send')
        self.assertIn('Check backlight permissions', result.stderr)

    def test_power_actions_and_unknown_selection(self):
        actions = {'Lock': ('swaylock', ['-f']), 'Sleep': ('systemctl', ['suspend']),
                   'Log out': ('swaymsg', ['exit']), 'Restart': ('systemctl', ['reboot']),
                   'Shut down': ('systemctl', ['poweroff'])}
        for choice in [*actions, 'Restart; echo injected', '']:
            with self.subTest(choice=choice):
                self.reset()
                result = self.execute('power-menu', responses={'fuzzel': {'out': choice+'\n'}})
                self.assertEqual(result.returncode, 0, result.stderr)
                actions_seen = [(e['name'], e['args']) for e in self.events()[1:]]
                self.assertEqual(actions_seen, [actions[choice]] if choice in actions else [])

    def test_launcher_search_and_data_paths(self):
        for inherited in ('', '/custom/share'):
            with self.subTest(inherited=inherited):
                self.reset()
                result = self.execute('launcher', '--width=42', XDG_DATA_DIRS=inherited)
                self.assertEqual(result.returncode, 0, result.stderr)
                event = self.events()[0]
                self.assertEqual(event['args'], ['--placeholder=Search apps', '--width=42'])
                self.assertTrue(event['data'].startswith(inherited or '/usr/local/share:/usr/share'))
                self.assertIn(str(self.home / '.local/share'), event['data'])
                self.assertIn('/var/lib/flatpak/exports/share', event['data'])

    def test_session_start_order_and_environment(self):
        result = self.execute('sway-session', responses={
            'systemctl --user --quiet is-active sway-session.target': {'code': 3}},
            XDG_CURRENT_DESKTOP='wrong', XDG_SESSION_TYPE='x11', TOP_SECRET='never-import')
        self.assertEqual(result.returncode, 0, result.stderr)
        events = self.events()
        self.assertEqual([e['name'] for e in events], ['systemctl', 'swaymsg', 'dbus-update-activation-environment',
                                                     'systemctl', 'systemctl', 'systemctl'])
        self.assertEqual(events[-1]['args'], ['--user', 'start', 'sway-session.target'])
        self.assertEqual(events[3]['args'], ['--user', 'list-units', '--failed', '--type=service', '--plain', '--no-legend', '--no-pager', 'kanshi.service', 'sway-session-monitor.service',
                                          'battery-monitor.service', 'swayidle.service', 'udiskie.service'])
        self.assertIn('WAYLAND_DISPLAY', events[4]['args'])
        self.assertNotIn('TOP_SECRET', events[4]['args'])
        self.assertTrue(all(e['desktop'] == 'sway' and e['session'] == 'wayland' for e in events))

    def test_session_resets_only_failed_session_units(self):
        result = self.execute('sway-session', responses={
            'systemctl --user --quiet is-active sway-session.target': {'code': 3},
            'systemctl --user list-units --failed --type=service --plain --no-legend --no-pager kanshi.service sway-session-monitor.service battery-monitor.service swayidle.service udiskie.service':
                {'out': 'kanshi.service loaded failed failed Monitor manager\n'}})
        self.assertEqual(result.returncode, 0, result.stderr)
        resets = [e for e in self.events() if 'reset-failed' in e['args']]
        self.assertEqual([e['args'] for e in resets], [['--user', 'reset-failed', 'kanshi.service']])

    def test_active_session_is_untouched(self):
        result = self.execute('sway-session')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.events()), 1)
        self.assertIn('already active', result.stderr)

    def test_failed_environment_import_prevents_target_start(self):
        result = self.execute('sway-session', responses={
            'systemctl --user --quiet is-active sway-session.target': {'code': 3},
            'dbus-update-activation-environment': {'code': 1}})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(len(self.events()), 3)

    def test_unavailable_compositor_prevents_session_start(self):
        result = self.execute('sway-session', responses={
            'systemctl --user --quiet is-active sway-session.target': {'code': 3},
            'swaymsg': {'code': 1}})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([e['name'] for e in self.events()], ['systemctl', 'swaymsg'])

    def test_notes_focus_existing_window_without_creating_note(self):
        tree = {'nodes': [{'app_id': 'notes'}]}
        result = self.execute('notes-toggle', responses={'swaymsg -t get_tree': {'out': json.dumps(tree)}})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse((self.home / 'vault').exists())
        self.assertEqual(self.events()[-1]['args'], ['[app_id="notes"] focus'])

    def test_notes_reject_invalid_compositor_response(self):
        for data in ('not JSON', 'null', '[]'):
            with self.subTest(data=data):
                self.reset()
                result = self.execute('notes-toggle', responses={'swaymsg -t get_tree': {'out': data}})
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse((self.home / 'vault').exists())
                self.assertFalse(any(e['name'] == 'foot' for e in self.events()))

    def test_zen_argument_forwarding_and_wayland(self):
        binary = self.home / '.local/zen/zen'
        binary.parent.mkdir(parents=True)
        binary.symlink_to(self.stub('zen-fixture'))
        result = self.execute('zen', 'https://example.invalid/a?x=1&y=2', 'two words')
        self.assertEqual(result.returncode, 0, result.stderr)
        event = self.events()[0]
        self.assertEqual(event['args'], ['https://example.invalid/a?x=1&y=2', 'two words'])
        self.assertEqual(event['wayland'], '1')
        self.assertEqual(event['portal'], '1')

    def test_keybind_help_closes_on_eof(self):
        result = run(str(REPO / 'dotfiles/.local/bin/keybinds'), input='', env=self.env)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('Super', result.stdout)
        self.assertEqual(self.events(), [])

    def test_screenshot_capture_failure_removes_partial_file(self):
        self.stub('grim', "Path(args[-1]).write_bytes(b'partial')\n")
        result = self.execute('screenshot', 'full', responses={'grim': {'code': 1}})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(list((self.home / 'Pictures').glob('*.png')), [])
        self.assertEqual([e['name'] for e in self.events()], ['grim'])

    def test_screenshot_selection_geometry_private_unique_files(self):
        self.stub('grim', "Path(args[-1]).write_bytes(b'PNG')\n")
        for _ in range(2):
            result = self.execute('screenshot', 'select', responses={'slurp': {'out': '10,20 30x40\n'}})
            self.assertEqual(result.returncode, 0, result.stderr)
        images = list((self.home / 'Pictures').glob('*.png'))
        self.assertEqual(len(images), 2)
        self.assertTrue(all(p.stat().st_mode & 0o777 == 0o600 for p in images))
        captures = [e for e in self.events() if e['name'] == 'grim']
        self.assertTrue(all(e['args'][:2] == ['-g', '10,20 30x40'] for e in captures))

    def test_screenshot_clipboard_failure_keeps_capture_without_success_notice(self):
        self.stub('grim', "Path(args[-1]).write_bytes(b'PNG')\n")
        result = self.execute('screenshot', 'full', responses={'wl-copy': {'code': 1}})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(len(list((self.home / 'Pictures').glob('*.png'))), 1)
        self.assertFalse(any(e['name'] == 'notify-send' for e in self.events()))

    def test_screenshot_invalid_mode_has_no_side_effects(self):
        result = self.execute('screenshot', 'invalid')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([e['name'] for e in self.events()], ['notify-send'])
        self.assertFalse((self.home / 'Pictures').exists())
