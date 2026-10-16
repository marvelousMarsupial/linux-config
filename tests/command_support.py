"""Run user commands with isolated homes and recorded external actions."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from support import REPO, run


class CommandSandbox(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='config tests & ')
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)
        self.bin = self.home / 'bin'
        self.bin.mkdir()
        self.log = self.home / 'events.jsonl'
        self.env = dict(os.environ, HOME=str(self.home), PATH=f'{self.bin}:/usr/bin:/bin',
                        TEST_LOG=str(self.log), TEST_RESPONSES='{}')
        # Never invoke the live compositor, power manager, network or clipboard.
        for name in ('fuzzel', 'foot', 'swaymsg', 'swaylock', 'systemctl', 'loginctl',
                     'dbus-update-activation-environment', 'notify-send', 'nmcli',
                     'bluetoothctl', 'wl-copy', 'grim', 'slurp'):
            self.stub(name)

    def stub(self, name, body=''):
        path = self.bin / name
        path.write_text('''#!/usr/bin/python3
import json, os, sys
from pathlib import Path
name = Path(sys.argv[0]).name
args = sys.argv[1:]
with open(os.environ['TEST_LOG'], 'a') as log:
    log.write(json.dumps({'name': name, 'args': args,
        'desktop': os.environ.get('XDG_CURRENT_DESKTOP'),
        'session': os.environ.get('XDG_SESSION_TYPE'),
        'data': os.environ.get('XDG_DATA_DIRS'),
        'wayland': os.environ.get('MOZ_ENABLE_WAYLAND'),
        'portal': os.environ.get('GTK_USE_PORTAL')}) + '\\n')
responses = json.loads(os.environ['TEST_RESPONSES'])
response = responses.get(name + ' ' + ' '.join(args), responses.get(name, {}))
''' + body + '''
sys.stdout.write(response.get('out', ''))
sys.stderr.write(response.get('err', ''))
sys.exit(response.get('code', 0))
''')
        path.chmod(0o755)
        return path

    def execute(self, name, *args, responses=None, **env):
        return run(str(REPO / 'dotfiles/.local/bin' / name), *args,
                   env=dict(self.env, TEST_RESPONSES=json.dumps(responses or {}), **env))

    def events(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []

    def reset(self):
        self.log.unlink(missing_ok=True)

