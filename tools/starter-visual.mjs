// Offline E2E: actual App/client -> closed Rust bridge. Without a native host
// provider, Start MUST be refused; this bridge never grants consent or spawns
// editable Python. Successful process roundtrips are covered in Rust tests.
// No hardware, application server, catalog, installation or remote code.
import { build } from 'vite';
import react from '@vitejs/plugin-react';
import { mkdtemp, writeFile, readFile, open } from 'node:fs/promises';
import { resolve } from 'node:path';
import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { setTimeout as delay } from 'node:timers/promises';

const root = resolve(import.meta.dirname, '..');
const temp = await mkdtemp('/tmp/opencode/starter-e2e-');
await writeFile(`${temp}/index.html`, '<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><div id="root"></div><script type="module" src="./entry.tsx"></script></html>');
await writeFile(`${temp}/core.ts`, `export const isTauri=()=>true; let id=0; const pending=new Map(); window.starterDeliver=(id,reply)=>{const p=pending.get(id);pending.delete(id);if(reply.error)p.reject(reply.error);else p.resolve(reply.result)};export function invoke(name,args){if(name!=='starter_local')return Promise.reject(Error('Forbidden IPC in fixture: '+name));return new Promise((resolve,reject)=>{const n=++id;pending.set(n,{resolve,reject});window.__starterBridge(JSON.stringify({id:n,request:args.request}));});}`);
await writeFile(`${temp}/entry.tsx`, `import React from ${JSON.stringify(`${root}/node_modules/react/index.js`)};import {createRoot} from ${JSON.stringify(`${root}/node_modules/react-dom/client.js`)};import {App} from ${JSON.stringify(`${root}/apps/desktop/src/app/App.tsx`)};import ${JSON.stringify(`${root}/apps/desktop/src/ui/tokens.css`)};import ${JSON.stringify(`${root}/apps/desktop/src/ui/base.css`)};import ${JSON.stringify(`${root}/apps/desktop/src/ui/details.css`)};createRoot(document.getElementById('root')).render(<App software={[]}/>);`);
await build({ configFile: false, root: temp, base: './', plugins: [react()], define: { 'import.meta.env.DEV': 'true' }, resolve: { alias: { '@tauri-apps/api/core': `${temp}/core.ts`, 'react/jsx-runtime': `${root}/node_modules/react/jsx-runtime.js` } }, build: { outDir: `${temp}/dist`, emptyOutDir: false } });
const host = spawn(`${root}/apps/desktop/src-tauri/target/debug/starter-check`, ['--rpc'], { stdio: ['pipe', 'pipe', 'pipe'] });
const replies = createInterface({ input: host.stdout });
const waiters = [];
let hostError = '';
host.stderr.on('data', (data) => { hostError += data; });
replies.on('line', (line) => waiters.shift()?.(JSON.parse(line)));
const hostRpc = (request) => Promise.race([
  new Promise((ok) => { waiters.push(ok); host.stdin.write(`${JSON.stringify(request)}\n`); }),
  delay(5000).then(() => { throw Error('Rust bridge timeout'); }),
]);
const log = await open(`${temp}/chromium.log`, 'w');
const browser = spawn('chromium', ['--headless', '--disable-gpu', '--disable-background-networking', '--no-first-run', '--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost', '--allow-file-access-from-files', '--remote-debugging-address=127.0.0.1', '--remote-debugging-port=0', `--user-data-dir=${temp}/profile`, `file://${temp}/dist/index.html`], { stdio: ['ignore', log.fd, log.fd] });
let ws;
const actions = [];
try {
  let port;
  for (let i = 0; i < 100; i++) {
    try { port = Number((await readFile(`${temp}/profile/DevToolsActivePort`, 'utf8')).split('\n')[0]); if (port) break; } catch {}
    await delay(50);
  }
  if (!port) throw Error('Chromium unavailable');
  const page = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((p) => p.type === 'page');
  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((ok, fail) => { ws.onopen = ok; ws.onerror = fail; });
  let sequence = 0;
  let bridgeTail = Promise.resolve();
  const pending = new Map();
  const rpc = (method, params = {}) => new Promise((ok, fail) => { const id = ++sequence; pending.set(id, (m) => m.error ? fail(m.error) : ok(m.result)); ws.send(JSON.stringify({ id, method, params })); });
  ws.onmessage = ({ data }) => {
    const message = JSON.parse(data);
    if (message.id) { pending.get(message.id)?.(message); pending.delete(message.id); }
    if (message.method === 'Runtime.bindingCalled') {
      const input = JSON.parse(message.params.payload);
      bridgeTail = bridgeTail.then(async () => {
        actions.push(input.request.action);
        const reply = await hostRpc(input.request);
        await rpc('Runtime.evaluate', { expression: `window.starterDeliver(${input.id},${JSON.stringify(reply)})` });
      });
    }
  };
  await rpc('Runtime.addBinding', { name: '__starterBridge' });
  const evaluate = async (expression) => { const r = await rpc('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true }); if (r.exceptionDetails) throw r.exceptionDetails; return r.result.value; };
  const wait = async (expression) => { for (let i = 0; i < 100; i++) { if (await evaluate(`!!(${expression})`)) return; await delay(50); } throw Error(`Timed out: ${expression}`); };
  const click = async (text) => { await evaluate(`(()=>{const b=Array.from(document.querySelectorAll('button')).find(b=>b.textContent.trim()===${JSON.stringify(text)});if(!b||b.disabled)throw Error('Unavailable button: '+${JSON.stringify(text)});b.click()})()`); await delay(80); };
  await wait('document.querySelector(".app-menu button")');
  await evaluate('document.querySelector(".menu-toggle").click()');
  await click('Registrar starter local (DEV)');
  await wait('document.querySelector("[data-software]")');
  if (actions.length) throw Error('Registration must not auto-start');
  await evaluate('document.querySelector("[data-software]").click()');
  const report = [];
  for (const [width, height] of [[1280, 720], [390, 844]]) {
    await rpc('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
    await click('Iniciar processo');
    await wait('document.querySelector(".starter-preview [role=alert]")?.textContent.includes("confirmação nativa")');
    const geometry = await evaluate('({overflow:document.documentElement.scrollWidth>innerWidth,disabled:document.querySelector("#setting-sample_value").disabled,status:document.querySelector(".starter-preview [role=status]").textContent,error:document.querySelector(".starter-preview [role=alert]").textContent})');
    if (geometry.overflow || !geometry.disabled || geometry.status.includes('Lido do processo:')) throw Error('Refusal/state/geometry failure');
    await evaluate('window.scrollTo(0,0)');
    const image = await rpc('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true });
    const path = `${temp}/${width}-consent-required.png`;
    await writeFile(path, Buffer.from(image.data, 'base64'));
    report.push({ width, height, path, ...geometry });
  }
  await click('Voltar');
  await wait('document.querySelector("[data-software]")===document.activeElement');
  await bridgeTail;
  const stopped = await hostRpc({ action: 'capabilities' });
  if (stopped.error?.kind !== 'NOT_STARTED') throw Error('Unexpected process after refused Start');
  const restarted = await hostRpc({ action: 'start' });
  if (restarted.error?.kind !== 'NATIVE_CONFIRMATION_REQUIRED') throw Error('CLI must never stand in for native consent');
  await hostRpc({ action: 'stop' });
  await writeFile(`${temp}/report.json`, JSON.stringify({ report, actions, stopped }, null, 2));
  console.log(JSON.stringify({ report, actions, stopped }, null, 2));
} finally {
  ws?.close();
  const exited = new Promise((ok) => browser.once('exit', ok));
  browser.kill('SIGTERM');
  await exited;
  replies.close();
  host.stdin.end();
  const hostExit = await new Promise((ok) => host.once('exit', ok));
  await log.close();
  if (hostExit !== 0 || hostError) throw Error(`Bridge failed: ${hostExit} ${hostError}`);
}
