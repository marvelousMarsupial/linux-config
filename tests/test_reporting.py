"""Display generation and read-only status parsing, including degraded input."""
import contextlib
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from support import load_script


class MonitorTests(unittest.TestCase):
    def setUp(self):
        self.monitor = load_script('monitor-info')

    def render(self, outputs):
        text = io.StringIO()
        with patch.object(self.monitor, 'get_outputs', return_value=outputs), contextlib.redirect_stdout(text):
            self.monitor.main()
        return text.getvalue()

    def test_docked_layout_disabled_output_and_escaping(self):
        text = self.render([
            {'name': 'eDP-1'},
            {'name': 'DP-1', 'make': 'A"B', 'model': 'Panel\\One\nTwo', 'serial': 'Unknown',
             'current_mode': {'width': 1920, 'height': 1080, 'refresh': 59950}},
            {'name': 'HDMI-A-1', 'current_mode': None,
             'modes': [{'width': 2560, 'height': 1440, 'refresh': 60000}]}])
        self.assertIn('output eDP-1 disable', text)
        self.assertIn('A\\"B Panel\\\\One Two', text)
        self.assertIn('position 0,0 mode 1920x1080@59.95Hz scale 1', text)
        self.assertIn('output "HDMI-A-1" enable position 1920,0 mode 2560x1440@60Hz', text)
        self.assertEqual(text.count('\n'), 6)

    def test_laptop_only_has_no_profile(self):
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as result:
            self.render([{'name': 'eDP-1'}])
        self.assertEqual(result.exception.code, 0)

    def test_ipc_failure_and_invalid_json_fail_cleanly(self):
        for response in ('garbage', '{}', 'null'):
            with self.subTest(response=response), patch.object(self.monitor.subprocess, 'run',
                    return_value=subprocess.CompletedProcess([], 0, response, '')):
                with self.assertRaises(SystemExit):
                    self.monitor.get_outputs()
        for error in (FileNotFoundError(), subprocess.TimeoutExpired('swaymsg', 5),
                      subprocess.CalledProcessError(1, 'swaymsg')):
            with self.subTest(error=error), patch.object(self.monitor.subprocess, 'run', side_effect=error), \
                 contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
                self.monitor.get_outputs()

    def test_missing_mode_does_not_emit_partial_profile(self):
        text = io.StringIO()
        with patch.object(self.monitor, 'get_outputs', return_value=[{'name': 'DP-1'}]), \
             contextlib.redirect_stdout(text), self.assertRaises(SystemExit):
            self.monitor.main()
        self.assertEqual(text.getvalue(), '')


class StatusTests(unittest.TestCase):
    def test_read_commands_are_bounded_and_tolerate_failures(self):
        for name, helper in (('motd', 'run_output'), ('status-osd', 'run_output'),
                             ('status-panel', 'run'), ('wifi-menu', 'run_output')):
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

    def test_real_nmcli_connection_types_and_escaped_names(self):
        osd = load_script('status-osd')
        cases = [(r'Cafe\:Lab:802-11-wireless', 'Cafe:Lab'),
                 ('LAN:802-3-ethernet', 'Ethernet'), ('Hotspot:wifi', 'Hotspot'),
                 ('bridge:bridge\nvpn:vpn', 'Offline'),
                 ('LAN:802-3-ethernet\nHome:802-11-wireless', 'Home'), ('', 'Offline')]
        for output, expected in cases:
            with self.subTest(output=output), patch.object(osd, 'run_output', return_value=output):
                self.assertEqual(osd.get_network(), expected)

    def test_bluetooth_reports_one_device_not_embedded_lines(self):
        osd = load_script('status-osd')
        with patch.object(osd, 'run_output', return_value='Device AA:BB Headphones\nDevice CC:DD Keyboard'):
            self.assertEqual(osd.get_bluetooth(), 'Headphones')

    def test_notification_contains_status_and_timeout(self):
        osd = load_script('status-osd')
        with patch('sys.argv', ['status-osd', '--timeout', '1234']), \
             patch.object(osd, 'get_battery', return_value='Battery 55% charging'), \
             patch.object(osd, 'get_network', return_value='Cafe:Lab'), \
             patch.object(osd, 'get_bluetooth', return_value='Headphones'), \
             patch.object(osd.subprocess, 'run') as command:
            osd.main()
        args = command.call_args.args[0]
        self.assertEqual(args[args.index('-t')+1], '1234')
        self.assertIn('Battery 55% charging\nCafe:Lab\nBluetooth Headphones', args[-1])


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
        for wifi, links, expected in (('Home', '', 'WiFi Home'),
                                       ('x'*50, '', 'WiFi '+'x'*19+'…'),
                                       ('', 'lo UP\nenx123 UP', 'Ethernet'),
                                       ('', 'lo UP\nwlan0 UP\nenp1 DOWN', 'Offline')):
            with self.subTest(wifi=wifi, links=links), patch.object(panel, 'run', side_effect=[wifi, links] if not wifi else [wifi]):
                self.assertEqual(panel.get_network(), expected)

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
