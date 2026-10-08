"""File-picker protocol checks without opening a terminal or browser."""
import json
from pathlib import Path
from command_support import CommandSandbox
from support import REPO, run


class PortalTests(CommandSandbox):
    def setUp(self):
        super().setUp()
        self.wrapper = REPO / 'dotfiles/.config/xdg-desktop-portal-termfilechooser/yazi-portal'
        self.env.pop("SWAYSOCK", None)
        self.output = self.home / 'selected'
        self.stub('foot', '''
for arg in args:
    if arg.startswith('--chooser-file=') and 'PICKED' in os.environ:
        Path(arg.split('=', 1)[1]).write_text(os.environ['PICKED'])
    if arg.startswith('--cwd-file='):
        Path(arg.split('=', 1)[1]).write_text(str(Path.home()))
''')

    def choose(self, multiple='0', directory='0', save='0', path=None, **env):
        return run(str(self.wrapper), multiple, directory, save,
                   path or str(self.home), str(self.output), '0',
                   env=dict(self.env, **env))

    def test_open_and_save_paths_are_literal_arguments(self):
        path = str(self.home / 'quotes " $(touch BAD) `touch BAD` ; café.txt')
        for save in ('0', '1'):
            with self.subTest(save=save):
                self.reset()
                result = self.choose(save=save, path=path, PICKED=path + '\n')
                self.assertEqual(result.returncode, 0, result.stderr)
                event = self.events()[0]
                self.assertEqual(event['args'][-2:], ['--', path])
                self.assertIn('yazi', event['args'])
                self.assertEqual(self.output.read_text(), path + '\n')
                self.assertFalse((self.home / 'BAD').exists())

    def test_single_and_multiple_selection(self):
        for multiple, expected in [('0', '/one\n'), ('1', '/one\n/two\n')]:
            result = self.choose(multiple=multiple, PICKED='/one\n/two\n')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(self.output.read_text(), expected)

    def test_picker_opacity_in_glass_and_solid_modes(self):
        state = self.home / '.config/theme/glass-mode'
        state.parent.mkdir(parents=True)
        for mode, alpha in [('on', '0.60'), ('off', '1.0')]:
            with self.subTest(mode=mode):
                self.reset()
                state.write_text(mode + '\n')
                result = self.choose()
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn('--override=colors.alpha=' + alpha,
                              self.events()[0]['args'])

    def test_file_cancellation_returns_no_selection(self):
        result = self.choose()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.output.read_text(), '')

    def test_directory_current_folder_fallback_and_cleanup(self):
        result = self.choose(directory='1')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.output.read_text(), str(self.home) + '\n')
        cwd_arg = next(arg for arg in self.events()[0]['args'] if arg.startswith('--cwd-file='))
        self.assertFalse(Path(cwd_arg.split('=', 1)[1]).exists())

    def test_terminal_close_does_not_accept_current_folder(self):
        result = self.choose(directory='1', TEST_RESPONSES=json.dumps({'foot': {'code': 1}}))
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.output.exists())

    def test_fullscreen_is_disabled_before_picker_and_restored_after(self):
        snapshot = {'type': 'root', 'nodes': [
            {'type': 'workspace', 'id': 3, 'nodes': [
                {'type': 'con', 'id': 10, 'focused': True, 'fullscreen_mode': 1}]}]}
        restored = json.loads(json.dumps(snapshot))
        restored['nodes'][0]['nodes'][0]['fullscreen_mode'] = 0
        self.stub('swaymsg', """
if args == ['-r', '-t', 'get_tree']:
    state = Path.home() / 'queried'
    response = {'out': os.environ['RESTORE_TREE'] if state.exists() else os.environ['FIRST_TREE']}
    state.touch()
else:
    response = {'out': '[{"success":true}]'}
""")
        result = self.choose(SWAYSOCK='/test/socket', FIRST_TREE=json.dumps(snapshot),
                             RESTORE_TREE=json.dumps(restored))
        self.assertEqual(result.returncode, 0, result.stderr)
        events = self.events()
        self.assertEqual([e['name'] for e in events],
                         ['swaymsg', 'swaymsg', 'foot', 'swaymsg', 'swaymsg'])
        self.assertEqual(events[1]['args'], ['-r', '[con_id=10] fullscreen disable'])
        self.assertEqual(events[-1]['args'], ['-r', '[con_id=10] fullscreen enable'])
