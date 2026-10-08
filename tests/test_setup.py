"""Editor installer download/execution boundaries using local release fixtures."""
import hashlib
import io
import json
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
from support import REPO, run


class DownloadSetupTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='editor setup & ')
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)
        self.bin = self.home / '.local/bin'
        self.bin.mkdir(parents=True)
        self.log = self.home / 'commands'
        self.stub('nvim', 'exit 1')
        self.stub('uname', 'echo x86_64')
        self.stub('sudo', 'echo unexpected-sudo >> "$SETUP_LOG"; exit 99')
        self.stub('curl', '''printf 'curl %s\\n' "$*" >> "$SETUP_LOG"
while [ "$#" -gt 0 ]; do
    if [ "$1" = -o ]; then shift; cp -- "$FIXTURE_ARCHIVE" "$1"; exit 0; fi
    shift
done
exit 1''')
        self.archive = self.home / 'release.tar.gz'
        self.env = dict(os.environ, HOME=str(self.home), PATH=f'{self.bin}:/usr/bin:/bin',
                        SETUP_LOG=str(self.log), FIXTURE_ARCHIVE=str(self.archive),
                        NVIM_VERSION='v0.0.1')
        # Isolate temporary extraction as well as installed files.
        self.tmp = self.home / 'tmp'
        self.tmp.mkdir()
        self.env['TMPDIR'] = str(self.tmp)

    def stub(self, name, body):
        path = self.bin / name
        path.write_text('#!/bin/sh\n' + body + '\n')
        path.chmod(0o755)

    def release(self, body):
        data = ('#!/bin/sh\n' + body + '\n').encode()
        with tarfile.open(self.archive, 'w:gz') as archive:
            info = tarfile.TarInfo('nvim-linux-x86_64/bin/nvim')
            info.mode = 0o755
            info.size = len(data)
            archive.addfile(info, io.BytesIO(data))
        self.env['NVIM_SHA256'] = hashlib.sha256(self.archive.read_bytes()).hexdigest()

    def setup(self):
        return run(str(REPO / 'scripts/setup-editor'), '--skip-apt', env=self.env)

    def test_verified_release_install_backup_and_temp_cleanup(self):
        self.release('printf "nvim %s\\n" "$*" >> "$SETUP_LOG"')
        existing = self.home / '.local/nvim'
        existing.mkdir()
        (existing / 'old').write_text('previous installation')
        result = self.setup()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((self.bin / 'nvim').is_symlink())
        self.assertEqual((self.bin / 'nvim').resolve(), self.home / '.local/nvim/bin/nvim')
        backup = list((self.home / '.local').glob('nvim.backup.*'))
        self.assertEqual(len(backup), 1)
        self.assertEqual((backup[0] / 'old').read_text(), 'previous installation')
        commands = self.log.read_text()
        self.assertIn('--proto =https --tlsv1.2 --fail --location --retry 3', commands)
        self.assertIn('+Lazy! install', commands)
        self.assertNotIn('unexpected-sudo', commands)
        self.assertTrue((self.home / '.config/nvim/init.lua').exists())
        self.assertEqual(list(self.tmp.iterdir()), [])

    def test_release_binary_version_failure_preserves_existing_install(self):
        self.release('exit 4')
        existing = self.home / '.local/nvim'
        existing.mkdir()
        (existing / 'old').write_text('keep')
        result = self.setup()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual((existing / 'old').read_text(), 'keep')
        self.assertFalse((self.bin / 'nvim').is_symlink())
        self.assertEqual(list(self.tmp.iterdir()), [])

    def test_plugin_failure_does_not_report_completion(self):
        self.release('''printf 'nvim %s\\n' "$*" >> "$SETUP_LOG"
case "$*" in *'Lazy! install'*) exit 7 ;; esac''')
        result = self.setup()
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('Editor setup complete', result.stdout)
        self.assertEqual(list(self.tmp.iterdir()), [])

    def test_version_and_digest_validation_happen_before_download(self):
        for version, digest in (('../bad', 'a'*64), ('v0.0.1', ''), ('v0.0.1', 'invalid')):
            with self.subTest(version=version, digest=digest):
                self.env.update(NVIM_VERSION=version, NVIM_SHA256=digest)
                result = self.setup()
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(self.log.exists())
                self.assertFalse((self.home / '.local/nvim').exists())

    def test_unsupported_architecture_does_not_download(self):
        self.stub('uname', 'echo unsupported')
        result = self.setup()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('for this architecture', result.stderr)
        self.assertFalse(self.log.exists())
