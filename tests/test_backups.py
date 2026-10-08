"""Optional backups must fail closed and preserve Restic failure status."""
from command_support import CommandSandbox


class BackupTests(CommandSandbox):
    def setUp(self):
        super().setUp()
        self.stub('restic')
        self.config = self.home / '.config/restic-home'
        self.config.mkdir(parents=True)
        self.password = self.config / 'password'
        self.password.write_text('test-secret\n')
        self.password.chmod(0o600)
        self.settings = self.config / 'env'
        self.configure('sftp:example:/backups/test')

    def configure(self, target):
        self.settings.write_text(f'RESTIC_REPOSITORY="{target}"\nRESTIC_PASSWORD_FILE="$HOME/.config/restic-home/password"\n')
        self.settings.chmod(0o600)

    def test_unconfigured_machine_never_contacts_target(self):
        self.settings.unlink()
        result = self.execute('restic-home', 'backup')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.events(), [])

    def test_home_and_exclusions_preserve_argument_boundaries(self):
        result = self.execute('restic-home', 'backup', '--dry-run')
        self.assertEqual(result.returncode, 0, result.stderr)
        args = self.events()[0]['args']
        self.assertEqual(args[-3:], ['--dry-run', '--', str(self.home)])
        self.assertIn(str(self.password), args)
        self.assertIn(str(self.config), args)
        self.assertIn(str(self.home / '.cache'), args)
        self.assertNotIn('test-secret', result.stdout + result.stderr + str(self.events()))

    def test_insecure_or_empty_credentials_fail_before_restic(self):
        for path in (self.settings, self.password):
            path.chmod(0o644)
            result = self.execute('restic-home', 'backup')
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(self.events(), [])
            path.chmod(0o600)
        self.password.write_text('')
        self.assertNotEqual(self.execute('restic-home', 'backup').returncode, 0)
        self.assertEqual(self.events(), [])

    def test_repository_inside_home_is_rejected_including_symlink(self):
        outside = self.home.parent / ('backup-link-' + self.home.name)
        outside.symlink_to(self.home, target_is_directory=True)
        self.addCleanup(outside.unlink)
        for target in (str(self.home / 'repo'), str(outside / 'repo')):
            self.configure(target)
            self.assertNotEqual(self.execute('restic-home', 'backup').returncode, 0)
            self.assertEqual(self.events(), [])

    def test_failure_propagates_without_pruning_or_initializing(self):
        result = self.execute('restic-home', 'backup', responses={'restic': {'code': 3}})
        self.assertEqual(result.returncode, 3)
        self.assertEqual(len(self.events()), 1)
        self.assertEqual(self.events()[0]['args'][0], 'backup')

    def test_explicit_init_and_restore_are_forwarded(self):
        for args in [('init',), ('restore', '12345678', '--target', '/tmp/restore test')]:
            self.reset()
            result = self.execute('restic-home', *args)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(self.events()[0]['args'], list(args))
