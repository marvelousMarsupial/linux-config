"""Fullscreen reveal/restore behavior using Sway IPC tree snapshots."""
import copy
import unittest
from support import load_script


def window(identifier, visible=False, fullscreen=0):
    return dict(id=identifier, type='con', app_id='fixture', visible=visible,
                fullscreen_mode=fullscreen, nodes=[], floating_nodes=[])


def tree(popups=(), floating=True, parent=True):
    workspace = dict(id=3, type='workspace', name='1', visible=True,
                     rect=dict(width=1280, height=800), nodes=[], floating_nodes=[])
    if parent:
        workspace['nodes'].append(window(10, visible=True, fullscreen=1))
    workspace['floating_nodes' if floating else 'nodes'].extend(copy.deepcopy(popups))
    return dict(id=1, type='root', nodes=[workspace], floating_nodes=[])


def event(change, target):
    return dict(change=change, container=dict(id=target))


class PopupGuardTests(unittest.TestCase):
    def setUp(self):
        self.module = load_script('sway-popup-guard')
        self.commands = []
        self.guard = self.module.PopupGuard(self.commands.append)

    def test_new_hidden_window_is_floated_centered_and_focused(self):
        self.guard.handle(event('new', 20), tree([window(20)], floating=False))
        self.assertEqual(len(self.commands), 1)
        self.assertIn('[con_id=10] fullscreen disable;', self.commands[0])
        self.assertIn('[con_id=20] floating enable, resize set 800 520, move position center, focus', self.commands[0])

    def test_native_visible_dialog_preserves_true_fullscreen(self):
        self.guard.handle(event('new', 20), tree([window(20, visible=True)]))
        self.assertEqual(self.commands, [])

    def test_delayed_visibility_on_focus_is_rechecked(self):
        self.guard.handle(event('new', 20), tree([window(20, visible=True)]))
        self.assertEqual(self.commands, [])
        self.guard.handle(event('focus', 20), tree([window(20)], floating=False))
        self.assertEqual(len(self.commands), 1)
        self.assertIn('[con_id=20] floating enable', self.commands[0])

    def test_hidden_floating_windows_moved_or_floated_are_revealed(self):
        for change in ('move', 'floating'):
            with self.subTest(change=change):
                commands = []
                guard = self.module.PopupGuard(commands.append)
                guard.handle(event(change, 20), tree([window(20)]))
                self.assertEqual(len(commands), 1)

    def test_startup_recovers_already_hidden_floaters(self):
        self.guard.recover(tree([window(20)]), startup=True)
        self.assertEqual(len(self.commands), 1)
        self.assertIn('[con_id=20] floating enable', self.commands[0])

    def test_startup_does_not_float_existing_tiled_windows(self):
        self.guard.recover(tree([window(20)], floating=False), startup=True)
        self.assertEqual(self.commands, [])

    def test_focus_does_not_float_preexisting_tiled_windows(self):
        self.guard.handle(event('focus', 20), tree([window(20)], floating=False))
        self.assertEqual(self.commands, [])

    def test_fullscreen_is_restored_after_last_popup_closes(self):
        self.guard.handle(event('new', 20), tree([window(20)]))
        after = tree([])
        after['nodes'][0]['nodes'][0]['fullscreen_mode'] = 0
        self.guard.handle(event('close', 20), after)
        self.assertEqual(self.commands[-1], '[con_id=10] fullscreen enable')
        self.assertEqual(self.guard.suspended, {})

    def test_duplicate_floating_event_does_not_resize_popup_again(self):
        self.guard.handle(event('new', 20), tree([window(20)]))
        self.guard.handle(event('floating', 20), tree([window(20, visible=True)]))
        self.assertEqual(len(self.commands), 1)

    def test_multiple_popups_delay_fullscreen_restoration(self):
        self.guard.handle(event('new', 20), tree([window(20)]))
        second = tree([window(20, visible=True), window(21, visible=True)])
        second['nodes'][0]['nodes'][0]['fullscreen_mode'] = 0
        self.guard.handle(event('new', 21), second)
        self.guard.handle(event('close', 20), second)
        self.assertEqual(len(self.commands), 2)
        self.guard.handle(event('close', 21), tree([]))
        self.assertEqual(self.commands[-1], '[con_id=10] fullscreen enable')

    def test_hidden_workspace_and_scratchpad_are_not_changed(self):
        for name, visible in (('2', False), ('__i3_scratch', True)):
            with self.subTest(name=name):
                snapshot = tree([window(20)])
                snapshot['nodes'][0].update(name=name, visible=visible)
                self.guard.handle(event('new', 20), snapshot)
        self.assertEqual(self.commands, [])

    def test_fullscreen_parent_focus_never_treats_it_as_popup(self):
        snapshot = tree([])
        self.guard.handle(event('new', 10), snapshot)
        self.guard.handle(event('focus', 10), snapshot)
        self.assertEqual(self.commands, [])

    def test_closed_parent_is_never_restored(self):
        self.guard.handle(event('new', 20), tree([window(20)]))
        self.guard.handle(event('close', 10), tree([window(20)], parent=False))
        self.guard.handle(event('close', 20), tree([], parent=False))
        self.assertEqual(len(self.commands), 1)

    def test_explicit_fullscreen_with_existing_floaters_is_respected(self):
        snapshot = tree([window(20)])
        self.guard.handle(event('fullscreen_mode', 10), snapshot)
        self.guard.recover(snapshot)
        self.guard.handle(event('focus', 20), snapshot)
        self.guard.recover(snapshot)
        self.assertEqual(self.commands, [])

    def test_parent_focus_rechecks_pending_new_window(self):
        self.guard.handle(event('new', 20), tree([window(20, visible=True)]))
        snapshot = tree([window(20)], floating=False)
        self.guard.handle(event('focus', 10), snapshot)
        self.guard.recover(snapshot)
        self.assertEqual(len(self.commands), 1)
        self.assertIn('[con_id=20] floating enable', self.commands[0])

    def test_fullscreen_after_popup_reveal_cancels_automatic_restoration(self):
        self.guard.handle(event('new', 20), tree([window(20)]))
        self.commands.clear()
        snapshot = tree([window(20)])
        self.guard.handle(event('fullscreen_mode', 10), snapshot)
        self.guard.recover(snapshot)
        self.guard.handle(event('focus', 20), snapshot)
        self.guard.handle(event('close', 20), tree([]))
        self.assertEqual(self.commands, [])
        self.assertEqual(self.guard.suspended, {})

    def test_pending_old_dialog_does_not_cancel_explicit_fullscreen(self):
        self.guard.handle(event('new', 20), tree([window(20, visible=True)]))
        snapshot = tree([window(20)])
        self.guard.handle(event('fullscreen_mode', 10), snapshot)
        for change in ('focus', 'title'):
            self.guard.handle(event(change, 20), snapshot)
            self.guard.recover(snapshot)
        self.assertEqual(self.commands, [])

    def test_new_popup_still_revealed_after_explicit_fullscreen(self):
        self.guard.handle(event('fullscreen_mode', 10), tree([window(20)]))
        self.guard.handle(event('new', 21), tree([window(20), window(21)]))
        self.assertEqual(len(self.commands), 1)
        self.assertIn('[con_id=21] floating enable', self.commands[0])

    def test_new_tiled_terminal_reported_visible_still_leaves_fullscreen(self):
        terminal = window(20, visible=True)
        terminal['app_id'] = 'foot'
        self.guard.handle(event('new', 20), tree([terminal], floating=False))
        self.assertEqual(len(self.commands), 1)
        self.assertIn('[con_id=10] fullscreen disable;', self.commands[0])
        self.assertIn('[con_id=20] floating enable', self.commands[0])

    def shortcut_terminal(self, identifier=20):
        popup = window(identifier, visible=True)
        popup['app_id'] = 'fullscreen-terminal-10-1'
        snapshot = tree([popup])
        snapshot['nodes'][0]['nodes'][0]['fullscreen_mode'] = 0
        return snapshot

    def test_shortcut_terminal_close_restores_original_fullscreen(self):
        snapshot = self.shortcut_terminal()
        self.guard.handle(event('new', 20), snapshot)
        self.assertEqual(self.commands, [])
        self.guard.handle(event('close', 20), snapshot)
        self.assertEqual(self.commands, ['[con_id=10] fullscreen enable'])

    def test_shortcut_terminal_move_to_other_monitor_restores_original(self):
        snapshot = self.shortcut_terminal()
        self.guard.handle(event('new', 20), snapshot)
        popup = snapshot['nodes'][0]['floating_nodes'].pop()
        other = tree([], parent=False)['nodes'][0]
        other.update(id=4, name='2', output='DP-2')
        other['floating_nodes'].append(popup)
        snapshot['nodes'].append(other)
        self.guard.handle(event('move', 20), snapshot)
        self.assertEqual(self.commands, ['[con_id=10] fullscreen enable'])
        self.assertEqual(self.guard.suspended, {})

    def test_shortcut_terminal_move_within_workspace_keeps_parent_unfullscreen(self):
        snapshot = self.shortcut_terminal()
        self.guard.handle(event('new', 20), snapshot)
        self.guard.handle(event('move', 20), snapshot)
        self.assertEqual(self.commands, [])

    def test_restart_recovers_shortcut_terminal_relationship(self):
        snapshot = self.shortcut_terminal()
        self.guard.recover(snapshot, startup=True)
        self.guard.handle(event('close', 20), snapshot)
        self.assertEqual(self.commands, ['[con_id=10] fullscreen enable'])

    def test_manual_fullscreen_does_not_retrack_shortcut_on_focus(self):
        snapshot = self.shortcut_terminal()
        self.guard.handle(event('new', 20), snapshot)
        snapshot['nodes'][0]['nodes'][0]['fullscreen_mode'] = 1
        self.guard.handle(event('fullscreen_mode', 10), snapshot)
        self.guard.handle(event('focus', 20), snapshot)
        self.guard.handle(event('close', 20), snapshot)
        self.assertEqual(self.commands, [])

    def test_global_shortcut_restores_global_fullscreen(self):
        snapshot = self.shortcut_terminal()
        snapshot['nodes'][0]['floating_nodes'][0]['app_id'] = 'fullscreen-terminal-10-2'
        self.guard.handle(event('new', 20), snapshot)
        self.guard.handle(event('close', 20), snapshot)
        self.assertEqual(self.commands, ['[con_id=10] fullscreen enable global'])


class TerminalShortcutTests(unittest.TestCase):
    def setUp(self):
        self.module = load_script('terminal')

    def snapshot(self, mode):
        snapshot = tree([])
        snapshot['nodes'][0]['nodes'][0].update(fullscreen_mode=mode, focused=True)
        return snapshot

    def test_fullscreen_terminal_exits_fullscreen_before_launch(self):
        import json
        from subprocess import CompletedProcess
        from unittest.mock import patch
        responses = [CompletedProcess([], 0, json.dumps(self.snapshot(1))),
                     CompletedProcess([], 0, '[{"success":true}]')]
        with patch.object(self.module.subprocess, 'run', side_effect=responses) as run, \
                patch.object(self.module.os, 'execvp') as launch:
            self.module.main()
        self.assertEqual(run.call_args_list[1].args[0],
                         ['swaymsg', '-r', '[con_id=10] fullscreen disable'])
        launch.assert_called_once_with('foot', ['foot', '--app-id=fullscreen-terminal-10-1'])

    def test_normal_terminal_stays_tiled(self):
        import json
        from subprocess import CompletedProcess
        from unittest.mock import patch
        with patch.object(self.module.subprocess, 'run',
                          return_value=CompletedProcess([], 0, json.dumps(self.snapshot(0)))) as run, \
                patch.object(self.module.os, 'execvp') as launch:
            self.module.main()
        self.assertEqual(run.call_count, 1)
        launch.assert_called_once_with('foot', ['foot'])

    def test_fullscreen_on_different_workspace_is_left_alone(self):
        snapshot = self.snapshot(1)
        snapshot['nodes'][0]['nodes'][0]['focused'] = False
        other = tree([], parent=False)['nodes'][0]
        other.update(id=4, name='2')
        other['nodes'].append(window(30, visible=True))
        other['nodes'][0]['focused'] = True
        snapshot['nodes'].append(other)
        self.assertIsNone(self.module.fullscreen_parent(snapshot))

    def test_global_fullscreen_is_detected_from_another_workspace(self):
        snapshot = self.snapshot(2)
        snapshot['nodes'][0]['nodes'][0]['focused'] = False
        self.assertEqual(self.module.fullscreen_parent(snapshot)['id'], 10)
