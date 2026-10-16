"""VPN menu: toggles, fail-off teardown, imports and session start."""
import json
from command_support import CommandSandbox

PROFILES = 'home:wireguard:no\nwork:wireguard:yes\nWired:802-3-ethernet:yes\n'
LIST = 'nmcli -t -f NAME,TYPE,ACTIVE connection show'


class VpnTests(CommandSandbox):
    def setUp(self):
        super().setUp()
        for name in ('tailscale', 'trash-put', 'xdg-open'):
            self.stub(name)

    def calls(self, name):
        return [e['args'] for e in self.events() if e['name'] == name]

    def test_menu_lists_states_and_connects_a_profile(self):
        responses = {LIST: {'out': PROFILES},
                     'tailscale status --json': {'out': json.dumps({'BackendState': 'Stopped'})},
                     'fuzzel': {'out': '0\n'}}
        result = self.execute('vpn-menu', responses=responses)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls('nmcli')[-1], ['connection', 'up', 'home'])
        self.assertEqual(self.calls('notify-send')[-1][-2:], ['VPN on', 'home'])

    def test_failed_connection_is_torn_down(self):
        responses = {LIST: {'out': PROFILES}, 'tailscale status --json': {'code': 1},
                     'fuzzel': {'out': '0\n'}, 'nmcli connection up home': {'code': 4, 'err': 'no route'}}
        result = self.execute('vpn-menu', responses=responses)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(self.calls('nmcli')[-2:], [['connection', 'up', 'home'], ['connection', 'down', 'home']])
        self.assertIn('-u', self.calls('notify-send')[-1])
        self.assertIn('no route', self.calls('notify-send')[-1][-1])

    def test_active_profile_disconnects_and_tailscale_rows_follow_state(self):
        responses = {LIST: {'out': PROFILES},
                     'tailscale status --json': {'out': json.dumps({'BackendState': 'Running'})},
                     'fuzzel': {'out': '1\n'}}
        self.assertEqual(self.execute('vpn-menu', responses=responses).returncode, 0)
        self.assertEqual(self.calls('nmcli')[-1], ['connection', 'down', 'work'])
        self.reset()
        responses['fuzzel'] = {'out': '2\n'}
        self.assertEqual(self.execute('vpn-menu', responses=responses).returncode, 0)
        self.assertEqual(self.calls('tailscale')[-1], ['down'])

    def test_tailscale_login_opens_browser_and_failure_fails_off(self):
        responses = {LIST: {'out': ''}, 'tailscale status --json': {'out': json.dumps({'BackendState': 'NeedsLogin'})},
                     'fuzzel': {'out': '0\n'},
                     'tailscale up --timeout=30s': {'code': 1, 'err': 'To authenticate, visit:\n\n\thttps://login.tailscale.com/a/abc123\n'}}
        self.assertEqual(self.execute('vpn-menu', responses=responses).returncode, 1)
        self.assertEqual(self.calls('xdg-open'), [['https://login.tailscale.com/a/abc123']])
        self.assertNotIn(['down'], self.calls('tailscale'))
        self.reset()
        responses['tailscale up --timeout=30s'] = {'code': 1, 'err': 'backend unavailable'}
        self.assertEqual(self.execute('vpn-menu', responses=responses).returncode, 1)
        self.assertEqual(self.calls('tailscale')[-1], ['down'])
        self.assertIn('-u', self.calls('notify-send')[-1])

    def test_import_disables_autoconnect_and_deletes_the_file(self):
        config = self.home / 'Downloads/home.conf'
        config.parent.mkdir()
        config.write_text('[Interface]\nPrivateKey = secret\n')
        responses = {f'nmcli connection import type wireguard file {config}':
                     {'out': "Connection 'home' (1234) successfully added.\n"}}
        result = self.execute('vpn-menu', 'import', str(config), responses=responses)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls('nmcli')[1:], [['connection', 'modify', 'home', 'connection.autoconnect', 'no'],
                                                   ['connection', 'down', 'home']])
        # The private key is in NetworkManager's store now; no readable copy may stay in home.
        self.assertFalse(config.exists())
        self.assertEqual(self.calls('trash-put'), [])
        self.assertIn('deleted', ' '.join(self.calls('notify-send')[-1]))
        self.reset()
        result = self.execute('vpn-menu', 'import', str(self.home / 'missing.conf'))
        self.assertEqual(result.returncode, 1)
        self.assertEqual(self.calls('nmcli'), [])

    def test_off_and_status_cover_wireguard_and_tailscale(self):
        responses = {LIST: {'out': PROFILES},
                     'tailscale status --json': {'out': json.dumps({'BackendState': 'Running'})}}
        result = self.execute('vpn-menu', 'status', responses=responses)
        self.assertEqual(result.stdout.split(), ['work', 'Tailscale'])
        self.reset()
        self.assertEqual(self.execute('vpn-menu', 'off', responses=responses).returncode, 0)
        self.assertEqual(self.calls('nmcli')[-1], ['connection', 'down', 'work'])
        self.assertEqual(self.calls('tailscale')[-1], ['down'])
        self.reset()
        responses['nmcli connection down work'] = {'code': 1}
        self.assertEqual(self.execute('vpn-menu', 'off', responses=responses).returncode, 1)

    def test_no_profiles_explains_instead_of_prompting(self):
        (self.bin / 'tailscale').unlink()
        result = self.execute('vpn-menu', responses={LIST: {'out': 'Wired:802-3-ethernet:yes\n'}})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls('fuzzel'), [])
        self.assertIn('No profiles', self.calls('notify-send')[-1][-1])
