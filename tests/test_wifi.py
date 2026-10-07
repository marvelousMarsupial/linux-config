"""Credential reuse, authentication retries and untrusted picker responses."""
import contextlib
import io
import subprocess
import unittest
from unittest.mock import call, patch
from support import load_script


class WifiTests(unittest.TestCase):
    def setUp(self):
        self.wifi = load_script('wifi-menu')

    def test_saved_names_unescape_without_stripping_ssid_spaces(self):
        with patch.object(self.wifi, 'run_output', return_value='Cafe\\:Lab\n spaced name \nback\\\\slash\n'):
            self.assertEqual(self.wifi.saved_connections(), {'Cafe:Lab', ' spaced name ', 'back\\slash'})

    def test_scan_caps_strongest_fifteen(self):
        sample = '\n'.join(f'network{i}:{i}:--' for i in range(30))
        with patch.object(self.wifi.subprocess, 'run'), patch.object(self.wifi, 'run_output', return_value=sample):
            result = self.wifi.scan_networks()
        self.assertEqual(len(result), 15)
        self.assertEqual([n[1] for n in result], list(range(29, 14, -1)))

    def test_open_and_saved_networks_do_not_prompt(self):
        for secured in (False, True):
            with self.subTest(secured=secured), patch.object(self.wifi, 'saved_connections', return_value={'Home'}), \
                 patch.object(self.wifi, 'nmcli_connect', return_value=(True, '')) as connect, \
                 patch.object(self.wifi, 'prompt_password') as prompt, patch.object(self.wifi, 'notify') as notify:
                self.wifi.connect('Home', secured)
                connect.assert_called_once_with('Home')
                prompt.assert_not_called()
                notify.assert_called_once_with('WiFi', 'Connected to Home')

    def test_saved_auth_failure_retries_once_with_replacement_password(self):
        with patch.object(self.wifi, 'saved_connections', return_value={'Home'}), \
             patch.object(self.wifi, 'nmcli_connect', side_effect=[(False, 'Secrets were required'), (True, '')]) as connect, \
             patch.object(self.wifi, 'prompt_password', return_value='new secret') as prompt, \
             patch.object(self.wifi, 'notify') as notify:
            self.wifi.connect('Home', True)
        self.assertEqual(connect.call_args_list, [call('Home'), call('Home', 'new secret')])
        prompt.assert_called_once_with('Home')
        notify.assert_called_once_with('WiFi', 'Connected to Home')

    def test_non_auth_failure_does_not_prompt(self):
        with patch.object(self.wifi, 'saved_connections', return_value={'Home'}), \
             patch.object(self.wifi, 'nmcli_connect', return_value=(False, 'No Wi-Fi device found')), \
             patch.object(self.wifi, 'prompt_password') as prompt, patch.object(self.wifi, 'notify') as notify:
            self.wifi.connect('Home', True)
        prompt.assert_not_called()
        self.assertTrue(notify.call_args.kwargs['critical'])

    def test_new_password_failure_does_not_claim_success(self):
        with patch.object(self.wifi, 'saved_connections', return_value=set()), \
             patch.object(self.wifi, 'nmcli_connect', return_value=(False, 'invalid password')), \
             patch.object(self.wifi, 'prompt_password', return_value='secret'), \
             patch.object(self.wifi, 'notify') as notify:
            self.wifi.connect('Home', True)
        self.assertIn('Failed to connect', notify.call_args.args[1])
        self.assertTrue(notify.call_args.kwargs['critical'])

    def test_password_masking_cancellation_and_whitespace(self):
        for code, output, expected in ((0, ' secret \n', ' secret '), (0, '\n', None), (1, 'secret\n', None)):
            with self.subTest(code=code, output=output), patch.object(self.wifi.subprocess, 'run',
                    return_value=subprocess.CompletedProcess([], code, output, '')) as command:
                self.assertEqual(self.wifi.prompt_password('Home'), expected)
                self.assertIn('--password=•', command.call_args.args[0])
                self.assertEqual(command.call_args.kwargs['timeout'], 120)

    def test_password_tool_absence_and_timeout_are_cancellation(self):
        for error in (FileNotFoundError(), subprocess.TimeoutExpired('fuzzel', 120)):
            with patch.object(self.wifi.subprocess, 'run', side_effect=error):
                self.assertIsNone(self.wifi.prompt_password('Home'))

    def test_connection_failures_return_diagnostics(self):
        for error, expected in ((FileNotFoundError(), 'nmcli is not installed'),
                                (subprocess.TimeoutExpired('nmcli', 45), 'connection timeout')):
            with patch.object(self.wifi.subprocess, 'run', side_effect=error):
                self.assertEqual(self.wifi.nmcli_connect('Home'), (False, expected))

    def test_menu_only_accepts_an_exact_successful_selection(self):
        networks = [('Cafe:Lab ', 80, True)]
        valid = '▂▄▆█ 󰌾 Cafe:Lab '
        for code, choice, accepted in ((0, valid, True), (1, valid, False), (0, 'Cafe:Lab ', False), (0, '', False)):
            with self.subTest(code=code, choice=choice), patch('sys.argv', ['wifi-menu']), \
                 patch.object(self.wifi, 'nm_running', return_value=True), \
                 patch.object(self.wifi, 'scan_networks', return_value=networks), \
                 patch.object(self.wifi.subprocess, 'run', return_value=subprocess.CompletedProcess([], code, choice+'\n', '')), \
                 patch.object(self.wifi, 'connect') as connect:
                try:
                    self.wifi.main()
                except SystemExit as result:
                    self.assertEqual(result.code, 0)
                self.assertEqual(connect.call_args_list, [call('Cafe:Lab ', True)] if accepted else [])

    def test_missing_manager_never_scans_or_connects(self):
        with patch('sys.argv', ['wifi-menu']), patch.object(self.wifi, 'nm_running', return_value=False), \
             patch.object(self.wifi, 'scan_networks') as scan, patch.object(self.wifi, 'notify'), \
             self.assertRaises(SystemExit) as result:
            self.wifi.main()
        self.assertEqual(result.exception.code, 1)
        scan.assert_not_called()

    def test_empty_scan_explains_adapter_or_wired_connection(self):
        for devices, expected in (('ethernet:connected', 'No WiFi - using Ethernet'),
                                  ('loopback:connected', 'No WiFi adapter'), ('wifi:disconnected', 'No networks found')):
            with self.subTest(devices=devices), patch('sys.argv', ['wifi-menu']), \
                 patch.object(self.wifi, 'nm_running', return_value=True), \
                 patch.object(self.wifi, 'scan_networks', return_value=[]), \
                 patch.object(self.wifi, 'run_output', return_value=devices), \
                 patch.object(self.wifi.subprocess, 'run') as command, self.assertRaises(SystemExit):
                self.wifi.main()
            self.assertEqual(command.call_args.kwargs['input'], expected)

    def test_help_does_not_query_services(self):
        with patch('sys.argv', ['wifi-menu', '--help']), patch.object(self.wifi, 'nm_running') as manager, \
             contextlib.redirect_stdout(io.StringIO()) as output:
            self.wifi.main()
        manager.assert_not_called()
        self.assertIn('USAGE', output.getvalue())
