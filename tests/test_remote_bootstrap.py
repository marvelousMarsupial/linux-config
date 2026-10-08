"""Exercise the piped bootstrap without network access or system changes."""
import os
from pathlib import Path
import tempfile
import unittest
from support import REPO, run


class RemoteBootstrapTests(unittest.TestCase):
    def test_piped_setup_arguments_failure_and_cleanup(self):
        for failure in ('none', 'setup', 'clone'):
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                binary = root / 'bin'
                binary.mkdir()
                git = binary / 'git'
                git.write_text('''#!/bin/bash
set -eu
for destination; do :; done
printf '%s' "$destination" > "$RECORD/checkout"
mkdir -p "$destination/scripts"
if [[ "$FAILURE" == clone ]]; then exit 42; fi
cat > "$destination/scripts/setup" <<'SETUP'
#!/bin/bash
printf '%s\\n' "$@" > "$RECORD/arguments"
if [[ "$FAILURE" == setup ]]; then exit 43; fi
SETUP
chmod +x "$destination/scripts/setup"
''')
                git.chmod(0o755)
                sudo = binary / 'sudo'
                sudo.write_text('#!/bin/sh\nexit 99\n')
                sudo.chmod(0o755)
                env = dict(os.environ, PATH=f'{binary}:/usr/bin:/bin',
                           RECORD=str(root), FAILURE=failure)
                result = run('bash', '-s', '--', '--dry-run', '--dev-tools',
                             input=(REPO / 'scripts/bootstrap').read_text(), env=env)
                self.assertEqual(result.returncode, {'none': 0, 'setup': 43, 'clone': 42}[failure], result.stderr)
                checkout = Path((root / 'checkout').read_text())
                self.assertFalse(checkout.parent.exists(), result.stderr)
                if failure != 'clone':
                    self.assertEqual((root / 'arguments').read_text(), '--dry-run\n--dev-tools\n')

    def test_fresh_install_receives_layout_and_keeps_local_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            result = run(str(REPO / 'scripts/install'), '--home', str(home))
            self.assertEqual(result.returncode, 0, result.stderr)
            config = home / '.config/kanshi/config'
            self.assertIn('position 0,740', config.read_text())
            self.assertIn('position 2560,0', config.read_text())
            config.write_text('# custom monitor layout\n')
            result = run(str(REPO / 'scripts/install'), '--home', str(home))
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(config.read_text(), '# custom monitor layout\n')

    def test_old_polkit_service_is_retired_with_backup(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            old = home / '.config/systemd/user/sway-polkit-agent.service'
            old.parent.mkdir(parents=True)
            old.write_text('[Service]\nExecStart=/old/agent\n')
            result = run(str(REPO / 'scripts/install'), '--home', str(home))
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(old.exists())
            backups = list((home / '.local/state/linux-config/backups').glob(
                '*/.config/systemd/user/sway-polkit-agent.service'))
            self.assertEqual(len(backups), 1)
            self.assertEqual(backups[0].read_text(), '[Service]\nExecStart=/old/agent\n')
