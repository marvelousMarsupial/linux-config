"""Completed update detection and delivery persistence."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from support import load_script


class UpdateNotificationTests(unittest.TestCase):
    def test_only_completed_successful_upgrades(self):
        module = load_script('update-notify')
        good = 'Start-Date: today\nUpgrade: example (1, 2)\nEnd-Date: today'
        self.assertEqual(len(module.completed_updates(good)), 1)
        for text in ('Upgrade: example (1, 2)', good + '\nError: failed',
                     'Install: example (1)\nEnd-Date: today'):
            self.assertFalse(module.completed_updates(text))

    def test_baseline_notify_once_and_retry_failure(self):
        module = load_script('update-notify')
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            log = root / 'history.log'
            log.write_text('Start-Date: old\nUpgrade: a (1, 2)\nEnd-Date: old\n\n')
            def mapped(value):
                return {'/var/log/apt/history.log': log,
                        '/var/log/apt/history.log.1.gz': root / 'absent.gz',
                        '/var/run/reboot-required': root / 'reboot-required'}.get(str(value), Path(value))
            with patch.object(module, 'Path', side_effect=mapped), \
                 patch.dict(os.environ, XDG_STATE_HOME=str(root)), \
                 patch.object(module.subprocess, 'run') as notify:
                module.main()
                notify.assert_not_called()
                with log.open('a') as stream:
                    stream.write('Start-Date: new\nUpgrade: b (1, 2)\nEnd-Date: new\n\n')
                notify.side_effect = subprocess.CalledProcessError(1, 'notify-send')
                with self.assertRaises(subprocess.CalledProcessError):
                    module.main()
                notify.side_effect = None
                (root / 'reboot-required').touch()
                module.main()
                self.assertIn('Restart when convenient', notify.call_args.args[0][-1])
                self.assertEqual(notify.call_count, 2)
                module.main()
                self.assertEqual(notify.call_count, 2)
