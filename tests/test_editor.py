"""Editor regression tests."""
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from support import REPO, run


@unittest.skipUnless(shutil.which('nvim') and (Path.home() / '.local/share/nvim/lazy/lazy.nvim').exists(),
                     'Neovim and installed plugins required')
class NeovimTests(unittest.TestCase):
    def test_operator_substitutions_and_toggle(self):
        result = run('nvim', '--headless', '-u', 'NONE', '-l', str(REPO / 'tests/nvim_ligatures.lua'))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('bold style and toggle passed', result.stderr)

    def test_filetypes_prose_saves_plugins_and_language_configs(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            config = root / 'config/nvim'
            shutil.copytree(REPO / 'dotfiles/.config/nvim', config)
            env = dict(os.environ, XDG_CONFIG_HOME=str(root / 'config'),
                       XDG_DATA_HOME=str(Path.home() / '.local/share'),
                       XDG_CACHE_HOME=str(root / 'cache'), XDG_STATE_HOME=str(root / 'state'),
                       LINUX_CONFIG_TEST_DIR=d,
                       LINUX_CONFIG_LSP_FIXTURE=str(REPO / "tests/fixtures/lsp_server.py"))
            result = run('nvim', '--headless', '-c',
                         f'lua dofile({json.dumps(str(REPO / "tests/nvim_smoke.lua"))})', '+qa', env=env)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertNotIn('Error', result.stderr)
            self.assertIn('ten filetypes', result.stderr)


class SetupTests(unittest.TestCase):
    def test_default_and_optional_install_branches(self):
        for development in (False, True):
            with self.subTest(development=development), tempfile.TemporaryDirectory() as d:
                home = Path(d)
                tools = home / '.local/bin'
                tools.mkdir(parents=True)
                log = home / 'commands.log'
                for name in ('sudo', 'nvim'):
                    stub = tools / name
                    stub.write_text('#!/bin/sh\nprintf "%s\\n" "' + name + ' $*" >> "$SETUP_LOG"\n')
                    stub.chmod(0o755)
                env = dict(os.environ, HOME=d, PATH=f'{tools}:/usr/bin:/bin', SETUP_LOG=str(log))
                args = ['--dev-tools'] if development else []
                result = run(str(REPO / 'scripts/setup-editor'), *args, env=env)
                self.assertEqual(result.returncode, 0, result.stderr)
                commands = log.read_text()
                self.assertIn('apt-get install -y jq bat', commands)
                self.assertEqual('apt-get install -y build-essential clangd clang-format ipython3 lazygit nodejs npm python3-matplotlib python3-numpy python3-scipy shellcheck' in commands, development)
                self.assertEqual('dofile(vim.env.LINUX_CONFIG_EDITOR_SETUP)' in commands, development)
                self.assertTrue((home / '.config/nvim/init.lua').exists())
