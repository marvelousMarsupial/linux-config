"""Shell helpers must preserve arguments and keep command secrets out of hints."""
import json
import fcntl
import os
import pty
import select
import subprocess
import termios
import time
from support import REPO, run
from command_support import CommandSandbox


class ShellTests(CommandSandbox):
    def zsh(self, script):
        return run('zsh', '-f', '-c', script, env=self.env)

    def test_arrows_search_the_typed_prefix_in_interactive_zsh(self):
        # Exercise real ZLE and the installed plugins, without executing history
        # entries or reading/writing the user's history.
        for keymap in ('emacs', 'viins'):
            for up, down in ((b'\x1b[A', b'\x1b[B'), (b'\x1bOA', b'\x1bOB')):
                with self.subTest(keymap=keymap, up=up):
                    setup = self.home / 'history-test.zsh'
                    setup.write_text(f'''source {json.dumps(str(REPO / 'dotfiles/.zshrc'))}
fc -p /dev/null
for entry in 'git status' 'echo unrelated' 'git stash list' 'git log' 'git status --short'; do
  print -s -- "$entry"
done
_capture() {{ print -r -- "CAPTURE:$BUFFER:END"; zle -R; }}
_reset() {{ zle .end-of-history; BUFFER=''; CURSOR=0; }}
zle -N _capture
zle -N _reset
bindkey -A {keymap} main
bindkey '^X' _capture
bindkey '^Y' _reset
PS1='history-test> '
export FZF_CTRL_R_OPTS='--filter="git status --short"'
print SETUP-DONE
''')
                    master, slave = pty.openpty()
                    process = subprocess.Popen(
                        ['zsh', '-f'], stdin=slave, stdout=slave, stderr=slave,
                        start_new_session=True,
                        preexec_fn=lambda: fcntl.ioctl(0, termios.TIOCSCTTY, 0),
                        env=dict(self.env, TERM='xterm', ZDOTDIR=str(self.home), XDG_VTNR='0'))
                    os.close(slave)

                    def read_until(marker):
                        output = b''
                        deadline = time.monotonic() + 10
                        while marker not in output:
                            self.assertLess(time.monotonic(), deadline, repr(output))
                            if select.select([master], [], [], 0.1)[0]:
                                output += os.read(master, 65536)
                        return output

                    def check(keys, expected):
                        os.write(master, keys + b'\x18')
                        output = read_until(b':END\r\n')
                        self.assertIn(b'CAPTURE:' + expected + b':END\r\n', output)

                    try:
                        os.write(master, f'source "{setup}"\n'.encode())
                        read_until(b'\r\nSETUP-DONE\r\n')
                        check(b'git st' + up, b'git status --short')
                        check(up, b'git stash list')
                        check(up, b'git status')
                        check(up, b'git status')  # Exhaustion keeps the match.
                        check(down, b'git stash list')
                        check(down, b'git status --short')
                        check(down, b'git st')  # Restore original input.
                        check(b'\x19git status -' + up, b'git status --short')
                        check(b'\x19stash' + up, b'stash')  # Match only at start.
                        check(b'\x19' + up, b'git status --short')
                        check(up, b'git log')  # Empty input browses everything.
                        check(down, b'git status --short')
                        check(down, b'')
                        check(b'\x12', b'git status --short')  # fzf Ctrl+R inserts, never executes.
                    finally:
                        process.kill()
                        process.wait(timeout=5)
                        os.close(master)

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
