"""Appearance palette previews and safe application of combined settings."""
from pathlib import Path
from types import SimpleNamespace
import shutil
import os
import re
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch
from support import REPO, load_script


class AppearanceTests(unittest.TestCase):
    def setUp(self):
        self.menu = load_script('appearance-menu')
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.home = Path(temporary.name)
        shutil.copytree(REPO / 'dotfiles/.config/theme', self.home / '.config/theme')
        current = self.home / '.cache/theme/current'
        current.parent.mkdir(parents=True)
        current.write_text('kanagawa\n')
        (self.home / '.config/theme/glass-mode').write_text('off\n')

    def apply(self, scheme, glass):
        with patch.object(self.menu.subprocess, 'run',
                          return_value=subprocess.CompletedProcess([], 0, '', '')) as run:
            self.menu.apply_settings(self.home, scheme, glass)
        return [call.args[0] for call in run.call_args_list]

    def test_combined_changes_apply_theme_before_glass(self):
        self.assertEqual(self.apply('summer-dusk', True), [
            [str(self.home / '.local/bin/theme'), 'apply', 'summer-dusk'],
            [str(self.home / '.local/bin/glass-mode'), 'on']])

    def test_unchanged_settings_do_not_write(self):
        self.assertEqual(self.apply('kanagawa', False), [])

    def test_glass_only_uses_explicit_state(self):
        self.assertEqual(self.apply('kanagawa', True), [
            [str(self.home / '.local/bin/glass-mode'), 'on']])
        (self.home / '.config/theme/glass-mode').write_text('on\n')
        self.assertEqual(self.apply('kanagawa', False), [
            [str(self.home / '.local/bin/glass-mode'), 'off']])

    def test_theme_only_preserves_glass(self):
        self.assertEqual(self.apply('summer-dusk', False), [
            [str(self.home / '.local/bin/theme'), 'apply', 'summer-dusk']])

    def test_no_current_theme_is_initialized_before_glass(self):
        (self.home / '.cache/theme/current').unlink()
        self.assertEqual(len(self.apply('summer-dusk', True)), 2)

    def test_invalid_selection_never_launches_command(self):
        with patch.object(self.menu.subprocess, 'run') as run:
            with self.assertRaises(ValueError):
                self.menu.apply_settings(self.home, '../../tmp/evil', True)
            run.assert_not_called()

    def test_theme_failure_prevents_glass_change(self):
        with patch.object(self.menu.subprocess, 'run',
                          return_value=subprocess.CompletedProcess([], 1, '', 'Render failed')) as run:
            with self.assertRaisesRegex(RuntimeError, 'Render failed'):
                self.menu.apply_settings(self.home, 'summer-dusk', True)
            self.assertEqual(run.call_count, 1)

    def test_invalid_palette_is_rejected(self):
        palette = self.home / '.config/theme/schemes/kanagawa.yaml'
        palette.write_text(palette.read_text().replace('1f1f28', 'invalid'))
        with self.assertRaisesRegex(ValueError, 'Invalid palette'):
            self.menu.load_palettes(self.home)

    def test_every_preview_stylesheet_parses_in_gtk(self):
        try:
            import gi
            gi.require_version('Gtk', '3.0')
            from gi.repository import Gtk
        except ImportError:
            self.skipTest('GTK introspection is not installed')
        palettes = self.menu.load_palettes(self.home)
        self.assertEqual(len(palettes), 6)
        for selected in palettes:
            with self.subTest(selected=selected):
                provider = Gtk.CssProvider()
                css = self.menu.stylesheet(palettes, selected)
                provider.load_from_data(css.encode())
                self.assertIn('#' + palettes[selected]['base00'], css)

    def test_combined_changes_render_real_outputs(self):
        config = self.home / '.config/theme/config.toml'
        config.write_text(re.sub(r'^hook = .*\n', '', config.read_text(), flags=re.M))
        binary = self.home / '.local/bin'
        binary.mkdir(parents=True)
        for name in ('theme', 'glass-mode'):
            shutil.copy2(REPO / 'dotfiles/.local/bin' / name, binary / name)
        with patch.dict(os.environ, HOME=str(self.home)):
            self.menu.apply_settings(self.home, 'summer-dusk', True)
            self.assertEqual(self.menu.get_state(self.home), ('summer-dusk', True))
            foot = self.home / '.config/foot/colors.ini'
            self.assertIn('alpha=0.0\n', foot.read_text())
            self.menu.apply_settings(self.home, 'kanagawa', False)
            self.assertEqual(self.menu.get_state(self.home), ('kanagawa', False))
            self.assertIn('alpha=1.0', foot.read_text())

    def controller(self):
        try:
            import gi
            gi.require_version('Gtk', '3.0')
            gi.require_version('Gdk', '3.0')
            from gi.repository import Gdk
        except ImportError:
            self.skipTest('GTK introspection is not installed')
        application = self.menu.create_application(self.home, self.menu.load_palettes(self.home))
        controller = SimpleNamespace(busy=False, switch=Mock(), rows=Mock(),
                                     window=Mock(), apply=Mock(), slugs=list(range(6)),
                                     theme_index=2, focus_row=Mock(), activate_selection=Mock())
        controller.rows.get_selected_row.return_value.get_index.return_value = 2
        return type(application), controller, Gdk

    def test_down_from_last_palette_selects_toggle_row(self):
        cls, controller, Gdk = self.controller()
        controller.rows.get_selected_row.return_value.get_index.return_value = 5
        cls.keypress(controller, None, Gdk.KEY_Down, 0, 0)
        controller.focus_row.assert_called_once_with(6)
        controller.switch.set_active.assert_not_called()

    def test_up_from_toggle_returns_to_last_palette(self):
        cls, controller, Gdk = self.controller()
        controller.rows.get_selected_row.return_value.get_index.return_value = 6
        cls.keypress(controller, None, Gdk.KEY_Up, 0, 0)
        controller.focus_row.assert_called_once_with(5)

    def test_enter_on_toggle_works_without_switch_having_widget_focus(self):
        cls, controller, Gdk = self.controller()
        controller.rows.get_selected_row.return_value.get_index.return_value = 6
        controller.switch.has_focus.return_value = False
        controller.switch.get_active.return_value = False
        controller.activate_selection = lambda: cls.activate_selection(controller)
        cls.keypress(controller, None, Gdk.KEY_Return, 0, 0)
        controller.switch.set_active.assert_called_once_with(True)
        controller.apply.assert_not_called()
        controller.window.close.assert_not_called()

    def test_enter_on_palette_applies_selection(self):
        cls, controller, Gdk = self.controller()
        controller.activate_selection = lambda: cls.activate_selection(controller)
        cls.keypress(controller, None, Gdk.KEY_Return, 0, 0)
        controller.apply.assert_called_once_with()

    def test_tab_and_arrows_use_same_toggle_selection(self):
        cls, controller, Gdk = self.controller()
        controller.rows.get_selected_row.return_value.get_index.return_value = 5
        cls.keypress(controller, None, Gdk.KEY_Tab, 0, 0)
        controller.focus_row.assert_called_once_with(6)
        controller.focus_row.reset_mock()
        controller.rows.get_selected_row.return_value.get_index.return_value = 6
        cls.keypress(controller, None, Gdk.KEY_Tab, 0, Gdk.ModifierType.SHIFT_MASK)
        controller.focus_row.assert_called_once_with(5)

    def test_toggle_preserves_current_palette_and_keeps_popup_open(self):
        cls, controller, _Gdk = self.controller()
        cls.toggle_transparency(controller)
        controller.apply.assert_called_once_with(close=False, scheme='kanagawa')

    def test_escape_closes_without_applying(self):
        cls, controller, Gdk = self.controller()
        cls.keypress(controller, None, Gdk.KEY_Escape, 0, 0)
        controller.window.close.assert_called_once_with()
        controller.apply.assert_not_called()

    def test_cards_use_each_palette_as_rendered_on_the_desktop(self):
        config = self.home / '.config/theme/config.toml'
        config.write_text(re.sub(r'^hook = .*\n', '', config.read_text(), flags=re.M))
        palettes = self.menu.load_palettes(self.home)
        for index, (slug, palette) in enumerate(palettes.items()):
            result = subprocess.run([str(REPO / 'dotfiles/.local/bin/theme'), 'apply', slug],
                                    env=dict(os.environ, HOME=str(self.home)),
                                    text=True, capture_output=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr)
            sway = (self.home / '.config/sway/colors').read_text()
            fuzzel = (self.home / '.config/fuzzel/colors.ini').read_text()
            background = re.search(r'^set \$base00 #(\w+)', sway, re.M)[1]
            text = re.search(r'^text=(\w{6})', fuzzel, re.M)[1]
            accent = re.search(r'^match=(\w{6})', fuzzel, re.M)[1]
            css = self.menu.stylesheet(palettes, slug)
            self.assertIn(f'#row-{index} {{ background: #{background}; color: #{text};', css)
            self.assertIn(f'#row-{index}:selected {{ border-color: #{accent}; }}', css)
            for chip, key in enumerate(self.menu.ACCENT_KEYS):
                self.assertIn(f'#chip-{index}-{chip} {{ background: #{palette[key]}; }}', css)

    def test_real_gtk_controller_and_toggle_activation(self):
        try:
            import gi
            gi.require_version('Gtk', '3.0')
            gi.require_version('Gdk', '3.0')
            from gi.repository import Gtk, Gdk, Gio
        except ImportError:
            self.skipTest('GTK introspection is not installed')
        if not Gtk.init_check()[0]:
            self.skipTest('No accessible display for real GTK keyboard routing')
        application = self.menu.create_application(self.home, self.menu.load_palettes(self.home))
        application.set_flags(Gio.ApplicationFlags.NON_UNIQUE)
        application.register(None)
        application.activate()
        self.addCleanup(application.window.destroy)
        while Gtk.events_pending():
            Gtk.main_iteration_do(False)
        self.assertEqual(application.keys.get_propagation_phase(), Gtk.PropagationPhase.CAPTURE)
        # The registered GTK signal, including its actual callback signature.
        application.focus_row(len(application.slugs) - 1)
        with patch.object(application, 'apply') as apply:
            application.keys.emit('key-pressed', Gdk.KEY_Down, 0, Gdk.ModifierType(0))
            self.assertEqual(application.rows.get_selected_row().get_index(), len(application.slugs))
            before = application.switch.get_active()
            application.keys.emit('key-pressed', Gdk.KEY_Return, 0, Gdk.ModifierType(0))
            self.assertEqual(application.switch.get_active(), not before)
            apply.assert_called_once_with(close=False, scheme='kanagawa')
