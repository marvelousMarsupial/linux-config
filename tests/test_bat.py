"""Bat regression tests."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from support import REPO, load_script, run


@unittest.skipUnless(shutil.which('batcat'), 'batcat required')
class BatTests(unittest.TestCase):
    def setUp(self):
        self.module = load_script('bat')
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)

    def frame(self, names, width=79, style='numbers,grid,header'):
        env = dict(os.environ, BAT_CONFIG_PATH='/dev/null')
        result = run('batcat', '--paging=never', '--color=always', '--decorations=always',
                     f'--terminal-width={width}', f'--style={style}', *map(str, names), env=env)
        self.assertEqual(result.returncode, 0, result.stderr)
        return list(self.module.fitted_frames(result.stdout.splitlines(keepends=True)))

    def test_fitted_frame_colors_and_unicode(self):
        path = self.home / 'short.txt'
        path.write_text('short\nlonger text\n界e\u0301\n')
        lines = self.frame([path])
        widths = [self.module.display_width(line.rstrip('\n')) for line in lines]
        self.assertEqual(len(set(widths)), 1, widths)
        self.assertLess(widths[0], 80)
        plain = [self.module.ANSI.sub('', line).rstrip('\n') for line in lines]
        self.assertTrue(plain[0].endswith('╮'))
        self.assertTrue(plain[-1].endswith('╯'))
        self.assertTrue(all(line.endswith('│') for line in plain[1:-1] if '├' not in line))
        self.assertIn('\x1b[', ''.join(lines))
        self.assertEqual(self.module.display_width('界e\u0301'), 3)

    def test_long_file_stays_inside_terminal(self):
        path = self.home / 'long.txt'
        path.write_text('x' * 300 + '\n')
        lines = self.frame([path], width=39)
        self.assertTrue(all(self.module.display_width(line.rstrip('\n')) == 40 for line in lines))

    def test_multiple_files_and_empty_file(self):
        paths = [self.home / name for name in ['a.txt', 'longer-filename.txt', 'empty.txt']]
        paths[0].write_text('a\n')
        paths[1].write_text('a bit longer\n')
        paths[2].write_text('')
        plain = ''.join(self.module.ANSI.sub('', line) for line in self.frame(paths))
        self.assertEqual(plain.count('╭'), 3)
        self.assertEqual(plain.count('╯'), 3)
        self.assertIn('a bit longer', plain)

    def test_piped_output_and_error_exit(self):
        path = self.home / 'raw.txt'
        path.write_text('keep\ttabs\nand content\n')
        result = run(str(REPO / 'dotfiles/.local/bin/bat'), str(path))
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, path.read_text())
        result = run(str(REPO / 'dotfiles/.local/bin/bat'), str(self.home / 'absent'))
        self.assertNotEqual(result.returncode, 0)

    def test_plain_input_and_large_spooled_file(self):
        self.assertEqual(list(self.module.fitted_frames(['plain\n'])), ['plain\n'])
        # Enough content to exceed the one-MiB memory buffer.
        lines = ['─────┬' + '─' * 70 + '\n', '     │ heading\n', '─────┼' + '─' * 70 + '\n']
        lines.extend(['   1 │ ' + 'x' * 60 + '\n'] * 18000)
        lines.append('─────┴' + '─' * 70 + '\n')
        result = list(self.module.fitted_frames(lines))
        self.assertEqual(len(result), len(lines))
        self.assertTrue(self.module.ANSI.sub('', result[-1]).rstrip().endswith('╯'))

    def test_terminal_pager_selection_argument_quoting_and_overrides(self):
        import pty
        tools = self.home / 'bin'
        tools.mkdir()
        fake = tools / 'batcat'
        fake.write_text('''#!/usr/bin/python3
import json, os, sys
print(json.dumps({'args': sys.argv[1:], 'pager': os.environ.get('BAT_PAGER'),
                  'paging': os.environ.get('BAT_PAGING')}))
''')
        fake.chmod(0o755)
        # Copy the wrapper to a path containing spaces to exercise pager quoting.
        wrapper = self.home / 'my bat wrapper'
        shutil.copy2(REPO / 'dotfiles/.local/bin/bat', wrapper)
        for override in ('', 'custom pager --flag'):
            with self.subTest(override=override):
                master, slave = pty.openpty()
                try:
                    env = dict(os.environ, PATH=f'{tools}:/usr/bin:/bin', BAT_PAGER=override,
                               BAT_PAGING='never', COLUMNS='80')
                    result = subprocess.run([str(wrapper), '--terminal-width=42', 'two words.txt'],
                                            stdout=slave, stderr=subprocess.PIPE, env=env, timeout=5)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    data = json.loads(os.read(master, 4096))
                    self.assertEqual(data['args'], ['--terminal-width=42', 'two words.txt'])
                    self.assertEqual(data['paging'], 'never')
                    if override:
                        self.assertEqual(data['pager'], override)
                    else:
                        import shlex
                        self.assertEqual(shlex.split(data['pager'])[1:], [str(wrapper), '--rounded-pager'])
                finally:
                    os.close(master)
                    os.close(slave)
