"""Real process lifecycle checks for the status popup toggle."""
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'dotfiles/.local/bin/status-panel'


class StatusToggleTests(unittest.TestCase):
    def test_show_is_idempotent_hide_and_toggle_close_only_panel(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stub = root / 'fuzzel'
            stub.write_text('#!/bin/sh\ncat >/dev/null\necho ready >> "$XDG_RUNTIME_DIR/opened"\nexec sleep 30\n')
            stub.chmod(0o755)
            env = dict(os.environ, XDG_RUNTIME_DIR=str(root), PATH=f'{root}:' + os.environ['PATH'])
            unrelated = subprocess.Popen(['sleep', '30'])
            self.addCleanup(lambda: (unrelated.terminate(), unrelated.wait()))
            for close in ('hide', 'toggle'):
                panel = subprocess.Popen([str(SCRIPT), 'show'], env=env)
                try:
                    deadline = time.monotonic() + 8
                    while not (root / 'opened').exists() and time.monotonic() < deadline:
                        time.sleep(.02)
                    self.assertTrue((root / 'opened').exists())
                    subprocess.run([str(SCRIPT), 'show'], env=env, check=True, timeout=3)
                    self.assertIsNone(panel.poll())
                    self.assertEqual((root / 'opened').read_text(), 'ready\n')
                    subprocess.run([str(SCRIPT), close], env=env, check=True, timeout=3)
                    self.assertEqual(panel.wait(timeout=3), 0)
                    self.assertIsNone(unrelated.poll())
                    (root / 'opened').unlink()
                finally:
                    if panel.poll() is None:
                        subprocess.run([str(SCRIPT), 'hide'], env=env, timeout=3)
                        panel.wait(timeout=3)
            subprocess.run([str(SCRIPT), 'hide'], env=env, check=True, timeout=3)
            self.assertFalse((root / 'opened').exists())
