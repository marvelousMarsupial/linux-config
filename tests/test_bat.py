"""Bat regression tests."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from support import REPO, load_script, run


class FrameTests(unittest.TestCase):
    """Frame rendering on bat-shaped input; no batcat needed."""
    def setUp(self):
        self.module = load_script('bat')

    def plain(self, lines):
        return [self.module.ANSI.sub('', line).rstrip('\n') for line in lines]

    def test_rules_become_rounded_corners_and_lines_get_a_right_edge(self):
        grid = ['─────┬' + '─' * 30 + '\n', '     │ File: a.txt\n', '─────┼' + '─' * 30 + '\n',
                '\x1b[38;5;238m   1\x1b[0m │ short\n', '   2 │ 界é\n', '─────┴' + '─' * 30 + '\n']
        lines = list(self.module.framed(grid))
        plain = self.plain(lines)
        # The header is the widest line at 18 cells; the frame stops one cell past it.
        self.assertEqual(plain[0], '     ╭' + '─' * 13 + '╮')
        self.assertEqual(plain[1], '     │ File: a.txt │')
        self.assertEqual(plain[2], '     ├' + '─' * 13 + '┤')
        self.assertEqual(plain[-1], '     ╰' + '─' * 13 + '╯')
        self.assertTrue(all(line.endswith('│') for line in plain[1:-1] if line[5] == '│'))
        self.assertEqual({self.module.display_width(line) for line in plain}, {20})
        self.assertIn('\x1b[38;5;238m', lines[3])
        self.assertEqual(self.module.display_width('界é'), 3)

    def test_rule_color_is_reused_for_the_edge_and_plain_text_passes_through(self):
        colored = ['\x1b[38;5;238m────┬────\x1b[0m\n', '    │ x\n', '\x1b[38;5;238m────┴────\x1b[0m\n']
        lines = list(self.module.framed(colored))
        self.assertTrue(lines[1].endswith('\x1b[38;5;238m│\x1b[0m\n'))
        self.assertEqual(list(self.module.framed(['plain\n', 'text\n'])), ['plain\n', 'text\n'])

    def test_full_width_text_keeps_a_blank_cell_before_the_edge(self):
        full = '  │ ' + 'x' * 32
        lines = self.plain(self.module.framed(['─' * 2 + '┬' + '─' * 33 + '\n', full + '\n', '──┴' + '─' * 33 + '\n']))
        self.assertEqual(lines[1], full + ' │')
        self.assertEqual(lines[0], '  ╭' + '─' * 34 + '╮')

    def test_each_file_gets_its_own_frame_width(self):
        grid = ['──┬────────────────\n', '  │ short\n', '──┴────────────────\n',
                'between\n',
                '──┬────────────────\n', '  │ a longer line\n', '──┴────────────────\n']
        lines = self.plain(self.module.framed(grid))
        self.assertEqual(lines[1], '  │ short │')
        self.assertEqual(lines[3], 'between')
        self.assertEqual(lines[5], '  │ a longer line │')

    def test_overlong_line_is_not_padded_negative(self):
        lines = list(self.module.framed(['──┬──\n', '  │ ' + 'x' * 20 + '\n', '──┴──\n']))
        self.assertTrue(lines[1].endswith('│\x1b[0m\n'))


@unittest.skipUnless(shutil.which('batcat'), 'batcat required')
class BatTests(unittest.TestCase):
    def setUp(self):
        self.module = load_script('bat')
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)

    def frame(self, names, width=79):
        env = dict(os.environ, BAT_CONFIG_PATH='/dev/null')
        result = run('batcat', '--paging=never', '--color=always', '--decorations=always',
                     f'--terminal-width={width}', '--style=numbers,grid,header', *map(str, names), env=env)
        self.assertEqual(result.returncode, 0, result.stderr)
        return list(self.module.framed(result.stdout.splitlines(keepends=True)))

    def test_wrapped_files_fill_the_terminal_and_short_files_hug_their_text(self):
        paths = [self.home / name for name in ['a.txt', 'long.txt']]
        paths[0].write_text('a\n')
        paths[1].write_text('x' * 300 + '\n')
        lines = self.frame(paths, width=39)
        plain = [self.module.ANSI.sub('', line.rstrip('\n')) for line in lines]
        tops = [i for i, line in enumerate(plain) if '╭' in line]
        self.assertEqual(len(tops), 2)
        self.assertEqual(sum('╯' in line for line in plain), 2)
        widths = [self.module.display_width(line) for line in plain]
        short, wrapped = widths[:tops[1]], widths[tops[1]:]
        self.assertEqual(set(wrapped), {41})
        self.assertEqual(len(set(short)), 1, short)
        self.assertLess(short[0], 41)

    def test_piped_output_and_error_exit(self):
        path = self.home / 'raw.txt'
        path.write_text('keep\ttabs\nand content\n')
        result = run(str(REPO / 'dotfiles/.local/bin/bat'), str(path))
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, path.read_text())
        result = run(str(REPO / 'dotfiles/.local/bin/bat'), str(self.home / 'absent'))
        self.assertNotEqual(result.returncode, 0)


class WrapperTests(unittest.TestCase):
    def test_terminal_arguments_pager_override_and_exit_status(self):
        import pty
        with tempfile.TemporaryDirectory() as d:
            tools = Path(d) / 'bin'
            tools.mkdir()
            fake = tools / 'batcat'
            fake.write_text('''#!/usr/bin/python3
import json, os, sys
print(json.dumps({'args': sys.argv[1:], 'pager': os.environ.get('BAT_PAGER')}))
sys.exit(3)
''')
            fake.chmod(0o755)
            wrapper = Path(d) / 'my bat wrapper'
            shutil.copy2(REPO / 'dotfiles/.local/bin/bat', wrapper)
            for override in ('', 'custom pager --flag'):
                with self.subTest(override=override):
                    master, slave = pty.openpty()
                    try:
                        env = dict(os.environ, PATH=f'{tools}:/usr/bin:/bin', BAT_PAGER=override, COLUMNS='80')
                        result = subprocess.run([str(wrapper), '-P', 'two words.txt'], stdout=slave,
                                                stderr=subprocess.PIPE, env=env, timeout=5)
                        self.assertEqual(result.returncode, 3, result.stderr)
                        data = json.loads(os.read(master, 4096))
                        if override:
                            self.assertEqual(data['args'], ['-P', 'two words.txt'])
                            self.assertEqual(data['pager'], override)
                        else:
                            self.assertEqual(data['args'][:3], ['--color=always', '--decorations=always', '--paging=never'])
                            self.assertEqual(data['args'][3:5], ['-P', 'two words.txt'])
                            self.assertEqual(data['args'][5], '--terminal-width=78')
                    finally:
                        os.close(master)
                        os.close(slave)
