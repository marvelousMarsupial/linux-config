#!/usr/bin/python3
"""Offline JSON-RPC fixture: initialization, diagnostics, hover and definition."""
import json
import sys


def send(message):
    body = json.dumps(dict(jsonrpc='2.0', **message)).encode()
    sys.stdout.buffer.write(f'Content-Length: {len(body)}\r\n\r\n'.encode() + body)
    sys.stdout.buffer.flush()


while True:
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            sys.exit(0)
        if line in (b'\r\n', b'\n'):
            break
        key, value = line.decode().split(':', 1)
        headers[key.lower()] = value.strip()
    request = json.loads(sys.stdin.buffer.read(int(headers['content-length'])))
    method = request.get('method')
    result = None
    if method == 'initialize':
        result = {'capabilities': {'textDocumentSync': 1, 'hoverProvider': True, 'definitionProvider': True}}
    elif method == 'textDocument/didOpen':
        uri = request['params']['textDocument']['uri']
        send({'method': 'textDocument/publishDiagnostics', 'params': {'uri': uri, 'diagnostics': [
            {'range': {'start': {'line': 0, 'character': 0}, 'end': {'line': 0, 'character': 3}},
             'severity': 2, 'source': 'fixture', 'message': 'fixture diagnostic'}]}})
    elif method == 'textDocument/hover':
        result = {'contents': {'kind': 'plaintext', 'value': 'fixture documentation'}}
    elif method == 'textDocument/definition':
        result = {'uri': request['params']['textDocument']['uri'], 'range': {
            'start': {'line': 0, 'character': 0}, 'end': {'line': 0, 'character': 3}}}
    elif method == 'exit':
        sys.exit(0)
    if 'id' in request:
        send({'id': request['id'], 'result': result})
