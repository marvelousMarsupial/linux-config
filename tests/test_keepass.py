"""KeePassXC window switching without reading or opening a real database."""
import json
from command_support import CommandSandbox


class KeePassTests(CommandSandbox):
    def setUp(self):
        super().setUp()
        self.stub('keepassxc')

    def toggle(self, nodes):
        tree = dict(type='root', nodes=nodes)
        return self.execute('keepass-toggle', responses={
            'swaymsg -r -t get_tree': {'out': json.dumps(tree)},
            'swaymsg': {'out': '[{"success":true}]'}})

    def workspace(self, name, focused=False, **window):
        return dict(type='workspace', name=name, nodes=[dict(
            id=20, type='con', app_id='org.keepassxc.KeePassXC', focused=focused,
            visible=True, **window)])

    def test_first_use_launches_without_database_or_password_arguments(self):
        result = self.toggle([])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.events()[-1], dict(name='keepassxc', args=[],
            desktop=self.env.get('XDG_CURRENT_DESKTOP'), session=self.env.get('XDG_SESSION_TYPE'),
            data=self.env.get('XDG_DATA_DIRS'), wayland=self.env.get('MOZ_ENABLE_WAYLAND'),
            portal=self.env.get('GTK_USE_PORTAL')))

    def test_focused_window_hides_without_locking_or_quitting(self):
        result = self.toggle([self.workspace('1', focused=True)])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.events()[-1]['args'], ['-r', '[con_id=20] move scratchpad'])

    def test_hidden_scratchpad_window_is_shown(self):
        result = self.toggle([self.workspace('__i3_scratch')])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.events()[-1]['args'], ['-r', '[con_id=20] scratchpad show, focus'])

    def test_visible_window_is_focused_without_launching_another(self):
        result = self.toggle([self.workspace('2')])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.events()[-1]['args'], ['-r', '[con_id=20] focus'])

    def test_bad_ipc_response_does_not_launch_application(self):
        result = self.execute('keepass-toggle', responses={'swaymsg': {'out': 'invalid'}})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([e['name'] for e in self.events()], ['swaymsg'])


class AppDarkTests(CommandSandbox):
    def test_launch_defaults_preserve_other_preferences_and_are_repeatable(self):
        from support import load_script
        module = load_script('app-dark-mode')
        for app, (rel, section, key, value) in module.THEMES.items():
            with self.subTest(app=app):
                path = self.home / rel
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(f'[{section}]\n{key}=light\ncustom=value\n[other]\nsetting=keep\n')
                module.apply(self.home, app)
                content = path.read_text()
                self.assertIn(f'{key}={value}', content)
                self.assertIn('custom=value', content)
                self.assertIn('[other]\nsetting=keep', content)
                module.apply(self.home, app)
                self.assertEqual(path.read_text(), content)
                self.assertEqual(path.stat().st_mode & 0o777, 0o600)
