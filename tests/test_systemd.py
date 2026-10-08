"""Exercise the installed IPC client against a disposable compositor socket."""
import json
import os
from pathlib import Path
import shutil
import socket
import struct
import subprocess
import tempfile
import unittest
from support import REPO


class SessionMonitorTests(unittest.TestCase):
    @unittest.skipUnless(shutil.which('swaymsg'), 'swaymsg is not installed')
    def test_monitor_subscribes_and_exits_on_compositor_disconnect(self):
        # Read the actual unit command so the quoting and payload are exercised.
        import shlex
        unit = REPO / 'dotfiles/.config/systemd/user/sway-session-monitor.service'
        command = next(line.split('=', 1)[1] for line in unit.read_text().splitlines()
                       if line.startswith('ExecStart='))
        with tempfile.TemporaryDirectory(prefix='sway-ipc-') as temporary:
            address = str(Path(temporary) / 'ipc.sock')
            with socket.socket(socket.AF_UNIX) as server:
                try:
                    server.bind(address)
                except PermissionError:
                    self.skipTest('sandbox prohibits local IPC sockets')
                server.listen(1)
                server.settimeout(5)
                process = subprocess.Popen(shlex.split(command),
                    env=dict(os.environ, SWAYSOCK=address),
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                try:
                    with server.accept()[0] as connection:
                        connection.settimeout(5)
                        def receive(size):
                            data = b''
                            while len(data) < size:
                                chunk = connection.recv(size - len(data))
                                if not chunk:
                                    self.fail('IPC client disconnected before subscribing')
                                data += chunk
                            return data
                        header = receive(14)
                        self.assertEqual(header[:6], b'i3-ipc')
                        length, message_type = struct.unpack('=II', header[6:])
                        self.assertEqual(message_type, 2)
                        self.assertEqual(json.loads(receive(length)), ['shutdown'])
                        payload = b'{"success":true}'
                        connection.sendall(b'i3-ipc' + struct.pack('=II', len(payload), 2) + payload)
                    # EOF must end the service, even without a graceful shutdown event.
                    process.communicate(timeout=5)
                    self.assertIsNotNone(process.returncode)
                finally:
                    if process.poll() is None:
                        process.kill()
                        process.communicate()
