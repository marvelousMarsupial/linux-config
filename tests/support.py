"""Shared helpers; scripts are imported without running their entry points."""
import importlib.machinery
import importlib.util
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
REPO = Path(__file__).resolve().parent.parent


def run(*args, env=None, **kwargs):
    return subprocess.run(args, env=env, capture_output=True, text=True, timeout=30, **kwargs)


def load_script(name):
    loader = importlib.machinery.SourceFileLoader(name, str(REPO / 'dotfiles/.local/bin' / name))
    spec = importlib.util.spec_from_loader(name, loader)
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    return module


