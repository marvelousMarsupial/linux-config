"""Display generation and read-only status parsing, including degraded input."""
import contextlib
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from support import REPO, load_script


class StatusTests(unittest.TestCase):
    def test_read_commands_are_bounded_and_tolerate_failures(self):
        for name, helper in (('motd', 'run_output'), ('status-panel', 'run'), ('wifi-menu', 'run_output')):
            module = load_script(name)
            for error in (FileNotFoundError(), subprocess.TimeoutExpired('tool', 10)):
                with self.subTest(name=name, error=error), patch.object(module.subprocess, 'run', side_effect=error) as cmd:
                    self.assertEqual(getattr(module, helper)(['missing']), '')
                    self.assertGreater(cmd.call_args.kwargs['timeout'], 0)
            with patch.object(module.subprocess, 'run', return_value=subprocess.CompletedProcess([], 1, 'bad data', '')):
                self.assertEqual(getattr(module, helper)(['missing']), '')

    def test_motd_memory_disk_and_malformed_input(self):
        motd = load_script('motd')
        for output, expected in (('Mem: 16Gi 4Gi 12Gi', '4Gi/16Gi'), ('Mem:', ''), ('', '')):
            with self.subTest(output=output), patch.object(motd, 'run_output', return_value=output):
                self.assertEqual(motd.get_memory(), expected)
        for output, expected in (('Filesystem Size Used\n/dev/vda 40G 12G', '12G/40G'),
                                 ('Filesystem\n/dev/vda', ''), ('', '')):
            with self.subTest(output=output), patch.object(motd, 'run_output', return_value=output):
                self.assertEqual(motd.get_disk(), expected)

    def test_motd_no_color_output(self):
        motd = load_script('motd')
        output = io.StringIO()
        with patch('sys.argv', ['motd', '--no-color']), patch.object(motd, 'run_output', return_value=''), \
             patch.object(motd, 'get_hostname', return_value='fixture-host'), contextlib.redirect_stdout(output):
            motd.main()
        self.assertIn('fixture-host', output.getvalue())
        self.assertNotIn('\x1b', output.getvalue())



class BatteryLoopTests(unittest.TestCase):
    def test_warning_deduplication_critical_and_charge_reset(self):
        battery = load_script('battery-monitor')
        levels = [21, 20, 19, 10, 9, 50, 20, 10, None]
        states = ['Discharging'] * 5 + ['Charging'] + ['Discharging'] * 3
        with patch('sys.argv', ['battery-monitor']), patch.object(battery, 'find_battery', return_value=Path('/fixture')), \
             patch.object(battery, 'get_battery_level', side_effect=levels), \
             patch.object(battery, 'get_battery_status', side_effect=states), \
             patch.object(battery.time, 'sleep') as sleep, patch.object(battery, 'notify') as notify, \
             self.assertRaises(SystemExit) as result:
            battery.main()
        self.assertEqual(result.exception.code, 0)
        self.assertEqual([c.args[0] for c in notify.call_args_list],
                         ['Battery low', 'Battery critical', 'Battery low', 'Battery critical'])
        self.assertTrue(notify.call_args_list[-1].kwargs['critical'])
        self.assertEqual(sleep.call_count, 8)

    def test_no_battery_exits_without_sleep_or_notification(self):
        battery = load_script('battery-monitor')
        with patch('sys.argv', ['battery-monitor']), patch.object(battery, 'find_battery', return_value=None), \
             patch.object(battery.time, 'sleep') as sleep, patch.object(battery, 'notify') as notify, \
             self.assertRaises(SystemExit):
            battery.main()
        sleep.assert_not_called()
        notify.assert_not_called()

    def test_notification_failure_is_nonfatal(self):
        battery = load_script('battery-monitor')
        for error in (FileNotFoundError(), subprocess.TimeoutExpired('notify-send', 5)):
            with patch.object(battery.subprocess, 'run', side_effect=error):
                battery.notify('Title', 'Message', critical=True)


class PanelTests(unittest.TestCase):
    def test_network_wifi_wired_and_offline(self):
        panel = load_script('status-panel')
        # nmcli is asked first (terse `ACTIVE:SSID` rows), then `ip -br link` when nothing is active.
        for wifi, links, expected in (('no:Other\nyes:Home', '', 'WiFi Home'),
                                       ('yes:' + 'x'*50, '', 'WiFi '+'x'*19+'…'),
                                       ('yes:Caf\\:Net', '', 'WiFi Caf:Net'),
                                       ('no:Other', 'lo UP\nenx123 UP', 'Ethernet'),
                                       ('', 'lo UP\nwlan0 UP\nenp1 DOWN', 'WiFi not managed'),
                                       ('', 'lo UP\nenp1 DOWN', 'Offline')):
            with self.subTest(wifi=wifi, links=links), patch.object(panel, 'run', side_effect=[wifi, links]):
                self.assertEqual(panel.get_network(), expected)

    def test_battery_found_by_type_not_name(self):
        panel = load_script('status-panel')
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            for name, kind, scope in (('AC', 'Mains', None), ('BAT1', 'Battery', 'System'),
                                      ('hidpp_battery_0', 'Battery', 'Device')):
                (root / name).mkdir()
                (root / name / 'type').write_text(kind + '\n')
                if scope:
                    (root / name / 'scope').write_text(scope + '\n')
            (root / 'BAT1' / 'capacity').write_text('57\n')
            (root / 'BAT1' / 'status').write_text('Discharging\n')
            real = panel.Path
            with patch.object(panel, 'Path', side_effect=lambda p: root if p == '/sys/class/power_supply' else real(p)):
                self.assertEqual(panel.get_power(), 'Battery 57%')
            (root / 'BAT1' / 'type').write_text('Mains\n')
            with patch.object(panel, 'Path', side_effect=lambda p: root if p == '/sys/class/power_supply' else real(p)):
                self.assertEqual(panel.get_power(), 'Power connected')

    def test_popup_is_read_only_and_has_no_application_search_label(self):
        panel = load_script('status-panel')
        with patch.object(panel, 'get_clock', return_value='clock one\nclock two'), \
             patch.object(panel, 'get_network', return_value='WiFi Home'), \
             patch.object(panel, 'get_power', return_value='Battery 50%'), \
             patch.object(panel.subprocess, 'run') as command:
            panel.main()
        args = command.call_args.args[0]
        self.assertIn('--dmenu', args)
        self.assertIn('--prompt=', args)
        self.assertNotIn('--placeholder=Search apps', args)
        text = command.call_args.kwargs['input']
        self.assertIn('WiFi Home', text)
        self.assertIn('Battery 50%', text)
        self.assertIn(f'--lines={len(text.splitlines())}', args)

    def test_popup_darkens_only_over_windows(self):
        import json
        import tempfile
        from pathlib import Path
        panel = load_script('status-panel')
        leaf = {'type': 'con', 'nodes': [], 'floating_nodes': [], 'app_id': 'foot'}
        workspace = lambda nodes: {'type': 'workspace', 'focused': True, 'nodes': nodes, 'floating_nodes': []}
        tree = lambda nodes: json.dumps({'type': 'root', 'nodes': [
            {'type': 'output', 'nodes': [workspace(nodes)], 'floating_nodes': []}], 'floating_nodes': []})
        with tempfile.TemporaryDirectory() as d:
            colors = Path(d) / '.config/fuzzel/colors.ini'
            colors.parent.mkdir(parents=True)
            colors.write_text('[colors]\nbackground=1f1d2e14\ntext=ffffffff\n')
            with patch.object(panel.Path, 'home', return_value=Path(d)):
                with patch.object(panel, 'run', return_value=tree([leaf])):
                    self.assertEqual(panel.covered_background(), ['--background=1f1d2ed9'])
                with patch.object(panel, 'run', return_value=tree([])):
                    self.assertEqual(panel.covered_background(), [])
                with patch.object(panel, 'run', return_value=''):
                    self.assertEqual(panel.covered_background(), [])


class MenuBackgroundTests(unittest.TestCase):
    def test_every_menu_darkens_over_windows_only_in_glass_mode(self):
        import json
        import tempfile
        from pathlib import Path
        helper = load_script('menu-background')
        leaf = {'type': 'con', 'nodes': [], 'floating_nodes': [], 'app_id': 'firefox-esr'}
        workspace = lambda nodes: {'type': 'workspace', 'focused': True, 'nodes': nodes, 'floating_nodes': []}
        tree = lambda nodes: json.dumps({'type': 'root', 'nodes': [
            {'type': 'output', 'nodes': [workspace(nodes)], 'floating_nodes': []}], 'floating_nodes': []})
        with tempfile.TemporaryDirectory() as d:
            home = Path(d)
            colors = home / '.config/fuzzel/colors.ini'
            colors.parent.mkdir(parents=True)
            colors.write_text('[colors]\nbackground=1f1d2e14\ntext=ffffffff\n')
            self.assertEqual(helper.covered_background(run=lambda cmd: tree([leaf]), home=home), ['--background=1f1d2ed9'])
            self.assertEqual(helper.covered_background(run=lambda cmd: tree([]), home=home), [])
            self.assertEqual(helper.covered_background(run=lambda cmd: '', home=home), [])
            # Glass off renders a solid background; a window underneath changes nothing.
            colors.write_text('[colors]\nbackground=1f1d2eff\n')
            self.assertEqual(helper.covered_background(run=lambda cmd: tree([leaf]), home=home), [])
            # No colors file at all (first login before theme apply) is not an error.
            colors.unlink()
            self.assertEqual(helper.covered_background(run=lambda cmd: tree([leaf]), home=home), [])
        # Every fuzzel menu consults the helper.
        for name in ('launcher', 'power-menu', 'notes-picker', 'notifications', 'wifi-menu', 'vpn-menu', 'status-panel'):
            text = (REPO / 'dotfiles/.local/bin' / name).read_text()
            self.assertTrue('menu-background' in text or 'menu_background' in text, name)


class HealthReportTests(unittest.TestCase):
    def test_report_escaping_isolation_modes_retention_and_database(self):
        from support import REPO
        result = subprocess.run(['bash', str(REPO / 'tests/test_sysreport.sh')], capture_output=True, text=True, timeout=120)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
