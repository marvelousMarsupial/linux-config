"""Project regression tests."""
import re
import unittest
from support import REPO, run


class ProjectTests(unittest.TestCase):
    def test_minimal_packages_and_retired_integrations(self):
        packages = {line for line in (REPO / 'packages.txt').read_text().splitlines()
                    if line and not line.startswith('#')}
        self.assertTrue({'ipython3', 'jq', 'bat', 'chromium'} <= packages)
        self.assertFalse(packages & {'podman', 'docker.io', 'incus', 'lxc', 'build-essential',
                                     'clangd', 'clang-format', 'nodejs', 'npm', 'tmux', 'wireguard-tools'})
        self.assertFalse((REPO / 'dotfiles/.config/nvim/lua/plugins/obsidian.lua').exists())
        self.assertFalse((REPO / 'dotfiles/.local/bin/vm-menu').exists())
        self.assertFalse((REPO / 'dotfiles/.local/bin/dictate').exists())
        self.assertFalse((REPO / 'dotfiles/.local/bin/whisper-server').exists())
        self.assertNotIn('get_stt', (REPO / 'dotfiles/.local/bin/status-panel').read_text())
        self.assertNotIn('SIGUSR1', (REPO / 'dotfiles/.config/theme/config.toml').read_text())

    def test_setup_cli_rejects_unknown_options(self):
        result = run(str(REPO / 'scripts/setup-editor'), '--help')
        self.assertEqual(result.returncode, 0)
        self.assertIn('--dev-tools', result.stdout)
        self.assertNotEqual(run(str(REPO / 'scripts/setup-editor'), '--nonsense').returncode, 0)

    def test_documentation_local_links_resolve(self):
        from urllib.parse import unquote
        files = [REPO / 'README.md', *sorted((REPO / 'docs').glob('*.md'))]
        for source in files:
            for link in re.findall(r'\[[^\]]*\]\(([^)]+)\)', source.read_text()):
                target = link.split('#', 1)[0]
                if not target or '://' in target or target.startswith('mailto:'):
                    continue
                with self.subTest(source=str(source.relative_to(REPO)), target=target):
                    self.assertTrue((source.parent / unquote(target)).exists())
