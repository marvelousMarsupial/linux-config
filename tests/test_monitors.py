"""monitor-setup fills kanshi profiles from the connected displays."""
import json
import shutil
from pathlib import Path
from support import REPO
from command_support import CommandSandbox


def output(name, make, model, serial, width=2560, height=1080, refresh=59997):
    return dict(name=name, make=make, model=model, serial=serial, active=True,
                current_mode=dict(width=width, height=height, refresh=refresh))


WQHD = output('DP-1', 'LG Electronics', 'LG HDR WQHD', '0A1B2C')
ULTRAWIDE = output('DP-2', 'LG Electronics', 'LG ULTRAWIDE', '9Z8Y7X', refresh=60000)


class MonitorSetupTests(CommandSandbox):
    def setUp(self):
        super().setUp()
        self.config = self.home / '.config/kanshi/config'
        self.config.parent.mkdir(parents=True)
        shutil.copy(REPO / 'dotfiles/.config/kanshi/config', self.config)
        self.stub('notify-send')

    def calls(self, name):
        return [e['args'] for e in self.events() if e['name'] == name]

    def setup(self, *args, outputs, **responses):
        responses = {'swaymsg -t get_outputs -r': {'out': json.dumps(outputs)}, **responses}
        return self.execute('monitor-setup', *args, responses=responses)

    def test_auto_fills_serials_and_modes_then_reloads_kanshi(self):
        result = self.setup('--auto', outputs=[WQHD, ULTRAWIDE])
        self.assertEqual(result.returncode, 0, result.stderr)
        text = self.config.read_text()
        self.assertNotIn('SERIAL1', text)
        self.assertNotIn('SERIAL2', text)
        self.assertIn('"LG Electronics LG HDR WQHD 0A1B2C" enable position 0,740  mode 2560x1080@59.997Hz transform normal', text)
        self.assertIn('"LG Electronics LG ULTRAWIDE 9Z8Y7X"  enable position 2560,0 mode 2560x1080@60Hz transform 90', text)
        self.assertEqual(text.count('0A1B2C'), 2)   # docked and only_curved
        self.assertIn('output eDP-1 enable position 0,0 scale 1', text)
        self.assertEqual(self.calls('systemctl'), [['--user', 'try-restart', 'kanshi.service']])
        self.assertIn('Profiles updated for DP-1, DP-2', ' '.join(self.calls('notify-send')[-1]))
        backups = list((self.home / '.local/state/linux-config/backups').glob('*/.config/kanshi/config'))
        self.assertEqual(len(backups), 1)
        self.assertIn('SERIAL1', backups[0].read_text())
        # A second run changes nothing and restarts nothing.
        self.reset()
        result = self.setup('--auto', outputs=[WQHD, ULTRAWIDE])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls('systemctl'), [])
        self.assertEqual(self.calls('notify-send'), [])

    def test_auto_skips_ambiguous_slots_and_the_picker_resolves_them(self):
        twin = output('DP-3', 'LG Electronics', 'LG ULTRAWIDE', '111111')
        result = self.setup('--auto', outputs=[WQHD, ULTRAWIDE, twin])
        self.assertEqual(result.returncode, 0, result.stderr)
        text = self.config.read_text()
        self.assertIn('0A1B2C', text)
        self.assertIn('SERIAL2', text)
        self.reset()
        prompt = 'fuzzel --dmenu --index --prompt=Which display is "LG Electronics LG ULTRAWIDE"?  --lines=2 --width=60'
        result = self.setup(outputs=[WQHD, ULTRAWIDE, twin], **{prompt: {'out': '1\n'}})
        self.assertEqual(result.returncode, 0, result.stderr)
        text = self.config.read_text()
        self.assertIn('"LG Electronics LG ULTRAWIDE 111111"', text)
        self.assertNotIn('SERIAL2', text)
        self.assertEqual(len(self.calls('fuzzel')), 1)

    def test_no_displays_or_cancel_leaves_the_config_alone(self):
        before = self.config.read_text()
        self.assertEqual(self.setup('--auto', outputs=[]).returncode, 0)
        self.assertNotEqual(self.setup(outputs=[]).returncode, 0)
        prompt = 'fuzzel --dmenu --index --prompt=Which display is "LG Electronics LG HDR WQHD"?  --lines=1 --width=60'
        other = output('HDMI-A-1', 'Dell', 'U2720Q', 'DELL1', 3840, 2160)
        result = self.setup(outputs=[other], **{prompt: {'code': 1}})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.config.read_text(), before)
        self.assertEqual(self.calls('systemctl'), [])
