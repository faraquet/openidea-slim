import base64
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import queue
import tomllib
import blake3

package = Path(__file__).resolve().parent.parent
server = Path(sys.argv[1]).resolve()
files = ['plugin.toml', 'languages/slim.toml', 'dist/plugin.wasm']
digest = blake3.blake3()
for name in sorted(files):
    digest.update(name.encode() + b'\0' + (package / name).read_bytes() + b'\0')

with tempfile.TemporaryDirectory(prefix='slim-runtime-') as tmp:
    root = Path(tmp)
    project = root / 'project'
    project.mkdir()
    sample = 'doctype html\nmain#content.card data-controller="booking"\n  - if @booking\n    h1 أهلاً #{user.name}\n    = render "booking", price: 12.50\n'
    for name in ['sample.slim', 'sample.html.slim', 'sample.html+mobile.slim', 'sample.html+desktop.slim']:
        (project / name).write_text(sample)
    env = dict(os.environ, OPENIDEA_SERVER_HOME=str(root / 'server'), OPENIDEA_SKIP_SHELL_ENV='1', OPENIDEA_TRUSTED_ROOT=str(project))
    log = open(root / 'runtime.log', 'w')
    p = subprocess.Popen([str(server), '--stdio'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log, env=env)
    messages = queue.Queue()
    def read():
        while True:
            headers = {}
            while True:
                line = p.stdout.readline()
                if not line:
                    messages.put({'error': 'server exited'})
                    return
                if line in (b'\r\n', b'\n'):
                    break
                k, v = line.decode().split(':', 1)
                headers[k.lower()] = v.strip()
            messages.put(json.loads(p.stdout.read(int(headers['content-length']))))
    threading.Thread(target=read, daemon=True).start()
    counter = 0
    def rpc(method, params):
        global counter
        counter += 1
        data = json.dumps(dict(jsonrpc='2.0', id=counter, method=method, params=params)).encode()
        p.stdin.write(f'Content-Length: {len(data)}\r\n\r\n'.encode() + data)
        p.stdin.flush()
        while True:
            result = messages.get(timeout=45)
            if result.get('id') == counter or 'error' in result:
                assert 'error' not in result, result
                return result['result']
    def invoke(cmd, **args):
        return rpc('invoke', dict(window='slim-test', cmd=cmd, args=args))
    try:
        print('Server:', rpc('initialize', {'version': 2}), flush=True)
        invoke('open_project', path=str(project))
        result = invoke('plugin_sync_packages', packages=[dict(id='faraquet.slim', digest=digest.hexdigest(), granted=[])], files={'faraquet.slim': [dict(path=name, data=base64.b64encode((package/name).read_bytes()).decode()) for name in files]})
        print('Plugin installation:', result, flush=True)
        assert result['installed'] == ['faraquet.slim'] and not result['failed'], result
        result = invoke('plugin_sync_languages', languages=[dict(plugin='faraquet.slim', language=tomllib.loads((package/'languages/slim.toml').read_text()), granted=[])])
        print('Language registration:', result, flush=True)
        assert result['languages'] == 1 and not result['rejected'], result
        invoke('open_project', path=str(project))
        for name in ['sample.slim', 'sample.html.slim', 'sample.html+mobile.slim', 'sample.html+desktop.slim']:
            doc = invoke('doc_open', path=str(project / name))
            assert doc['language'] == 'slim', doc
            assert doc['uiMode'] is None, doc
            result = invoke('doc_highlights', doc=doc['id'])
            data = result['data']
            kinds = result['kinds']
            encoded = sample.encode('utf-16-le')
            spans = [(encoded[a*2:b*2].decode('utf-16-le'), kinds[k]) for a,b,k in zip(data[::3], data[1::3], data[2::3])]
            for expected in [('main', 'tag'), ('#content', 'attribute'), ('data-controller', 'attribute'), ('if', 'keyword'), ('@booking', 'variable'), ('name', 'functionCall'), ('12.50', 'number')]:
                assert expected in spans, (name, expected, spans)
            print(f'PASS {name}: {len(spans)} highlight spans, expected categories and Unicode offsets verified', flush=True)
    finally:
        p.stdin.close()
        try:
            p.wait(timeout=15)
        except subprocess.TimeoutExpired:
            p.terminate()
            p.wait(timeout=5)
        log.close()
