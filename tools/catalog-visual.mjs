// Local-only visual fixture. Does not launch Tauri, enumerate USB or use HID.
// Requires existing workspace dependencies and Chromium; installs nothing.
import { build } from 'vite';
import react from '@vitejs/plugin-react';
import { mkdir, writeFile, readFile, open } from 'node:fs/promises';
import { resolve } from 'node:path';
import { spawn } from 'node:child_process';
import { setTimeout as delay } from 'node:timers/promises';

const root = resolve(import.meta.dirname, '..');
const temp = '/tmp/opencode/catalog-visual';
await mkdir(temp, { recursive: true });
const original = JSON.parse(await readFile(`${root}/tools/tests/fixtures/publication-base.json`, 'utf8'));
const fixture = { ...original, id: 'test.synthetic', name: 'Fixture sintética — não é software real', shortDescription: 'Dados de teste locais para revisar somente o catálogo.', fullDescription: 'Demonstração sintética. Não constitui aprovação, suporte físico ou software instalável.', version: '0.0.0-fixture', authors: ['Fixture sintética, não é pessoa'], license: 'Fixture, sem afirmação de licença', architectures: ['fixture'], devices: [{ manufacturer: 'Fabricante fictício', model: 'Modelo sintético', category: 'mouse', compatibility: 'Nenhum hardware real.' }], source: { repository: 'https://example.org/fixture', commit: 'a'.repeat(40) }, features: [{ name: 'Função sintética', implementation: 'not-implemented', verification: 'not-tested', notes: 'Não existe implementação física.' }], requirements: ['Nenhum hardware'], limitations: ['Fixture somente visual'] };
await writeFile(`${temp}/index.html`, '<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><div id="root"></div><script type="module" src="./entry.tsx"></script></html>');
await writeFile(`${temp}/core.ts`, `export const isTauri=()=>false; export async function invoke(command){window.fixtureRequests.push(command); if(command!=='catalog_sync')throw Error('Forbidden in fixture'); if(window.fixtureState==='error')return {index:null,status:'NETWORK',cacheWarning:null,checkedAt:null}; return {index:{schemaVersion:1,revision:'visual-fixture',software:window.fixtureState==='empty'?[]:[${JSON.stringify(fixture)}]},status:'UPDATED',cacheWarning:null,checkedAt:1};}`);
await writeFile(`${temp}/entry.tsx`, `import React from ${JSON.stringify(`${root}/node_modules/react/index.js`)}; import {createRoot} from ${JSON.stringify(`${root}/node_modules/react-dom/client.js`)}; import {App} from ${JSON.stringify(`${root}/apps/desktop/src/app/App.tsx`)}; import ${JSON.stringify(`${root}/apps/desktop/src/ui/tokens.css`)}; import ${JSON.stringify(`${root}/apps/desktop/src/ui/base.css`)}; import ${JSON.stringify(`${root}/apps/desktop/src/ui/details.css`)}; window.fixtureRequests=[];window.fixtureState='populated';createRoot(document.getElementById('root')).render(<App software={[]}/>);`);
await build({ configFile: false, root: temp, base: './', plugins: [react()], resolve: { alias: { '@tauri-apps/api/core': `${temp}/core.ts`, 'react/jsx-runtime': `${root}/node_modules/react/jsx-runtime.js` } }, build: { outDir: `${temp}/dist`, emptyOutDir: true } });
const log = await open(`${temp}/chromium.log`, 'w');
const browser = spawn('chromium', ['--headless', '--disable-gpu', '--disable-background-networking', '--no-first-run', '--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost', '--allow-file-access-from-files', '--remote-debugging-address=127.0.0.1', '--remote-debugging-port=9253', `--user-data-dir=${temp}/profile`, `file://${temp}/dist/index.html`], { stdio: ['ignore', log.fd, log.fd] });
let ws;
try {
  let page;
  for (let i = 0; i < 100; i++) {
    try { page = (await (await fetch('http://127.0.0.1:9253/json')).json()).find((p) => p.type === 'page'); if (page) break; } catch {}
    await delay(100);
  }
  if (!page) throw Error('Chromium did not expose local CDP');
  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((ok, fail) => { ws.onopen = ok; ws.onerror = fail; });
  let sequence = 0;
  const pending = new Map();
  ws.onmessage = ({ data }) => { const m = JSON.parse(data); if (m.id) { pending.get(m.id)?.(m); pending.delete(m.id); } };
  const rpc = (method, params = {}) => new Promise((ok, fail) => { const id = ++sequence; pending.set(id, (m) => m.error ? fail(m.error) : ok(m.result)); ws.send(JSON.stringify({ id, method, params })); });
  const evaluate = async (expression) => { const r = await rpc('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true }); if (r.exceptionDetails) throw r.exceptionDetails; return r.result.value; };
  const wait = async (expression) => { for (let i = 0; i < 100; i++) { if (await evaluate(`!!(${expression})`)) return; await delay(50); } throw Error(`Timed out: ${expression}`); };
  const click = async (text) => { await evaluate(`(()=>{const b=Array.from(document.querySelectorAll('button')).find(b=>b.textContent.trim()===${JSON.stringify(text)});if(!b||b.disabled)throw Error('Unavailable button');b.click()})()`); await delay(100); };
  await wait('document.querySelector(".page-navigation")');
  await click('Explorar softwares');
  await wait('document.querySelector(".catalog-list")');
  const report = [];
  for (const [width, height] of [[1366, 742], [900, 600], [390, 844]]) {
    await rpc('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
    await delay(100);
    for (const state of ['populated', 'details', 'error', 'empty']) {
      if (state === 'details') { await evaluate('document.querySelector(".catalog-list button").click()'); await wait('document.querySelector(".catalog-details")'); }
      else {
        if (await evaluate('!!document.querySelector(".catalog-details")')) await click('Voltar aos softwares');
        await evaluate(`window.fixtureState=${JSON.stringify(state)}`);
        await click('Atualizar catálogo');
      }
      await evaluate('window.scrollTo(0,0)');
      const geometry = await evaluate('({overflow:document.documentElement.scrollWidth>innerWidth,focus:document.activeElement?.textContent,requests:fixtureRequests,status:document.querySelector(".catalog [role=status]")?.textContent})');
      if (geometry.overflow) throw Error(`Horizontal overflow ${width} ${state}`);
      const image = await rpc('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true });
      const path = `${temp}/${width}-${state}.png`;
      await writeFile(path, Buffer.from(image.data, 'base64'));
      report.push({ width, height, state, path, ...geometry });
    }
    await evaluate("window.fixtureState='populated'"); await click('Atualizar catálogo');
  }
  if (!(await evaluate("fixtureRequests.every(c=>c==='catalog_sync')"))) throw Error('Unexpected command');
  await writeFile(`${temp}/report.json`, JSON.stringify(report, null, 2));
  console.log(`12 captures and geometry report: ${temp}/report.json`);
} finally { ws?.close(); browser.kill(); await log.close(); }
