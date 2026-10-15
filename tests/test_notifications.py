"""Exercise the notification viewer with fake D-Bus and fuzzel processes."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[1]


class NotificationsTests(unittest.TestCase):
    def run_viewer(self, entries, fail=False, cancel=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            payload = {'data': [[{
                key: {'type': 's', 'data': value} for key, value in item.items()
            } for item in entries]], 'type': 'aa{sv}'}
            (root / 'payload').write_text(json.dumps(payload))
            (root / 'busctl').write_text('''#!/bin/sh
if [ "$FAIL" = 1 ]; then exit 1; fi
case "$*" in
  *ListHistory) cat "$FIXTURE/payload" ;;
  *) printf '%s' '{"type":"aa{sv}","data":[[]]}' ;;
esac
''')
            (root / 'fuzzel').write_text('''#!/bin/sh
cat >> "$FIXTURE/display"
printf '\\n---\\n' >> "$FIXTURE/display"
case "$*" in
  *--index*)
    if [ "$CANCEL" = 1 ] || [ -f "$FIXTURE/chosen" ]; then exit 1; fi
    touch "$FIXTURE/chosen"
    printf '0\\n' ;;
  *) exit 1 ;;
esac
''')
            for name in ('busctl', 'fuzzel'):
                (root / name).chmod(0o755)
            result = subprocess.run([str(REPO / 'dotfiles/.local/bin/notifications')],
                                    env=dict(os.environ, PATH=f'{root}:' + os.environ['PATH'],
                                             FIXTURE=str(root), FAIL=str(int(fail)),
                                             CANCEL=str(int(cancel))), capture_output=True, text=True)
            return result, (root / 'display').read_text()

    def test_search_and_read_full_body_without_control_injection(self):
        result, display = self.run_viewer([{'app-name': 'Mail', 'summary': 'Hello',
                                           'body': 'First\nSecond\x00icon\x1fTest'}])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('[History] Mail — Hello — First Second icon Test', display)
        self.assertIn('Mail · History\n\nHello', display)
        self.assertNotIn('\x00', display)

    def test_empty_history(self):
        result, display = self.run_viewer([])
        self.assertEqual(result.returncode, 0)
        self.assertIn('No notifications yet', display)

    def test_session_failure_is_not_reported_as_empty(self):
        result, display = self.run_viewer([], fail=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn('Cannot read notifications', display)
        self.assertNotIn('No notifications yet', display)

    def test_cancel_does_not_open_detail(self):
        result, display = self.run_viewer([{'summary': 'Hello'}], cancel=True)
        self.assertEqual(result.returncode, 0)
        self.assertEqual(display.count('---'), 1)
