"""Shell helpers must preserve arguments and keep command secrets out of hints."""
import json
from support import REPO, run
from command_support import CommandSandbox


class ShellTests(CommandSandbox):
    def zsh(self, script):
        return run('zsh', '-f', '-c', script, env=self.env)

    def test_remote_terminfo_helper_rejects_options_and_requires_one_host(self):
        self.stub('ssh')
        self.stub('infocmp')
        source = REPO / 'dotfiles/.zsh_aliases'
        for args in ('', '-oProxyCommand=bad', 'host extra'):
            with self.subTest(args=args):
                result = self.zsh(f'source {json.dumps(str(source))}\nfix {args}')
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.events(), [])

    def test_remote_terminfo_helper_uses_end_of_options(self):
        self.stub('ssh')
        self.stub('infocmp')
        result = self.zsh(f'source {json.dumps(str(REPO / "dotfiles/.zsh_aliases"))}\nfix user@example.invalid')
        self.assertEqual(result.returncode, 0, result.stderr)
        ssh = next(e for e in self.events() if e['name'] == 'ssh')
        self.assertEqual(ssh['args'], ['--', 'user@example.invalid',
                                     'mkdir -p ~/.terminfo && tic -x -o ~/.terminfo /dev/stdin'])

    def test_alias_reminder_does_not_print_sensitive_arguments(self):
        source = REPO / 'dotfiles/.zsh_alias_reminder'
        result = self.zsh(f'''export ALIAS_REMINDER=1
alias gs='git status'
source {json.dumps(str(source))}
_ar_check 'git status --secret=do-not-display'
wait
''')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('gs', result.stderr)
        self.assertNotIn('do-not-display', result.stderr + result.stdout)
        # The notifier is deliberately detached. Inspect it through a synchronous hint
        # stub below rather than racing its log here.

    def test_alias_reminder_matches_single_word_and_longest_prefix(self):
        source = REPO / 'dotfiles/.zsh_alias_reminder'
        result = self.zsh(f'''export ALIAS_REMINDER=1
alias g=git gs='git status' c=clear
source {json.dumps(str(source))}
_ar_hint() {{ print -r -- "$1"; }}
_ar_check clear
_ar_check 'git status --short'
_ar_check 'git clone example'
''')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.splitlines(), ['c', 'gs', 'g'])

    def test_yazi_editor_opener_handles_spaces_and_option_like_filename(self):
        import tomllib
        self.stub('nvim')
        config = tomllib.loads((REPO / 'dotfiles/.config/yazi/yazi.toml').read_text())
        opener = config['opener']['edit'][0]
        self.assertTrue(opener['block'])
        result = run('sh', '-c', opener['run'], 'yazi', '-option.txt', 'folder/file with spaces.md', env=self.env)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.events()[0]['args'], ['--', '-option.txt', 'folder/file with spaces.md'])
