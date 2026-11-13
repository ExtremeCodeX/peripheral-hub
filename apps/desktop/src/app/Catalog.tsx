import { useEffect, useRef, useState } from 'react';
import { CatalogClient, catalogMessages, type CatalogSoftware } from './catalog';
import './catalog.css';
import { DeviceCardFrame } from '../ui/DeviceCard';
import { CoverageBadge } from '../ui/SoftwareStatus';
import { X6CardArt } from '../integrations/x6/MouseArt';
import type { PublicRelease } from './distribution';

const normalize = (text: string) => text.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLocaleLowerCase().trim();

export function useCatalog(active: boolean) {
  const [, render] = useState(0);
  const service = useRef<CatalogClient | null>(null);
  if (!service.current) service.current = new CatalogClient(() => render((v) => v + 1));
  const client = service.current;
  useEffect(() => () => client.dispose(), [client]);
  useEffect(() => { if (active) void client.enter(); }, [active, client]);
  return client;
}

export function Catalog({ client, query, releases = [], installing = false, installationError = '', onInstall, onOpen }: { client: CatalogClient; query: string; releases?: PublicRelease[]; installing?: boolean; installationError?: string; onInstall?: (id: string) => void; onOpen?: () => void }) {
  const [selected, setSelected] = useState<CatalogSoftware | null>(null);
  const detailRef = useRef<HTMLHeadingElement>(null);
  const trigger = useRef<HTMLButtonElement | null>(null);
  const entries = client.result.index?.software ?? [];
  const visible = entries.filter((entry) => normalize([entry.name, ...entry.devices.flatMap((d) => [d.model, d.manufacturer])].join(' ')).includes(normalize(query)));
  const current = selected && entries.find((entry) => entry.id === selected.id);
  useEffect(() => { setSelected(null); }, [query]);
  useEffect(() => { if (selected) detailRef.current?.focus(); }, [selected]);
  useEffect(() => {
    if (selected && !current) { setSelected(null); document.getElementById('collection-title')?.focus(); }
  }, [selected, current]);
  const message = catalogMessages[client.result.status] ?? catalogMessages.WORKER_ERROR;
  return <div className="catalog">
    {!!releases.length && <section aria-label="Versões públicas instaláveis">
      <h3>Versões públicas · Linux</h3>
      {installationError && <p role="alert">{installationError}</p>}
      <div className="device-grid">{releases.filter((r) => normalize(`${r.name} ${r.manufacturer}`).includes(normalize(query))).map((release) => <article key={release.id}>
        <DeviceCardFrame name={release.name} manufacturer={`${release.manufacturer} · Mouse`} visual={<X6CardArt/>} status={<CoverageBadge coverage="partial" label="Parcial" title="Recursos físicos reais; cobertura e validação parciais."/>} disabled={installing} aria-label={`${release.installed ? 'Abrir' : 'Instalar'} ${release.name}`} onClick={() => release.installed ? onOpen?.() : onInstall?.(release.id)}/>
        <p>{release.description}</p><p>Linux · {release.version} · {release.installed ? 'Instalado' : installing ? 'Instalando…' : 'Clique no card para instalar'}</p>
        <p>{release.connection}</p><ul>{release.limitations.map((limit) => <li key={limit}>{limit}</li>)}</ul>
        <p><a href={release.repository} target="_blank" rel="noreferrer">Repositório</a> · <a href={`${release.repository}/releases/tag/v${release.version}`} target="_blank" rel="noreferrer">Release {release.version}</a></p>
        <small>Download fixado por manifesto/SHA-256. Não é sandbox; iniciar e acessar hardware exigem consentimento GTK. Sem vídeo público de qualificação individual.</small>
      </article>)}</div>
    </section>}
    {(!releases.length || client.result.index || client.result.status !== 'NOT_CONFIGURED') && <><div className="collection-toolbar catalog-toolbar">
      <div role="status" aria-live="polite" aria-busy={client.busy}>
        <p>{client.busy ? 'Verificando catálogo…' : message}</p>
        {client.result.index && !['UPDATED', 'NOT_MODIFIED'].includes(client.result.status) && <p>Exibindo a última versão válida. Nenhum software local foi alterado.</p>}
        {client.result.cacheWarning && <p>O cache local não pôde ser lido ou salvo. {client.result.index ? 'A versão válida em memória foi preservada.' : 'Nenhuma versão inválida foi utilizada.'}</p>}
      </div>
      <button className="secondary-button" type="button" disabled={client.busy} onClick={() => void client.refresh()}>{client.busy ? 'Atualizando…' : 'Atualizar catálogo'}</button>
    </div>
    {client.busy && !client.result.index ? <div className="empty-state"><h3>Carregando catálogo…</h3><p>Validando os metadados antes de exibir os softwares.</p></div> : current ? <article className="catalog-details">
      <button type="button" className="text-button" onClick={() => { setSelected(null); requestAnimationFrame(() => trigger.current?.focus()); }}>Voltar aos softwares</button>
      <h3 ref={detailRef} tabIndex={-1}>{current.name}</h3>
      <p>{current.fullDescription}</p>
      <dl><dt>Versão</dt><dd>{current.version}</dd><dt>Autores</dt><dd>{current.authors.join(', ')}</dd><dt>Licença declarada</dt><dd>{current.license}</dd><dt>Arquiteturas declaradas</dt><dd>{current.architectures.join(', ')}</dd><dt>Commit da fonte</dt><dd>{current.source.commit}</dd></dl>
      <h4>Compatibilidade documentada</h4>
      {current.devices.map((d, i) => <p key={i}>{d.manufacturer} · {d.model}: {d.compatibility}</p>)}
      <h4>Funções declaradas</h4><ul>{current.features.map((f, i) => <li key={i}><strong>{f.name}</strong> — {f.implementation === 'implemented' ? 'Implementada' : f.implementation === 'partial' ? 'Parcial' : 'Não implementada'}. {f.notes}</li>)}</ul>
      <h4>Requisitos</h4><ul>{current.requirements.map((r) => <li key={r}>{r}</li>)}</ul>
      <h4>Limitações</h4><ul>{current.limitations.map((r) => <li key={r}>{r}</li>)}</ul>
      <p>Somente consulta de metadados. Instalação, atualização e execução de integrações não estão disponíveis neste catálogo.</p>
    </article> : visible.length ? <ul className="catalog-list" aria-label="Softwares do catálogo">{visible.map((entry) => <li key={entry.id}><button type="button" onClick={(event) => { trigger.current = event.currentTarget; setSelected(entry); }}><span><strong>{entry.name}</strong><span>{entry.shortDescription}</span><span>{entry.devices.map((d) => `${d.manufacturer} · ${d.model}`).join('; ')}</span></span><span>{entry.version}<span>Ver metadados</span></span></button></li>)}</ul> : <div className="empty-state"><h3>{entries.length ? 'Nenhum software encontrado' : client.result.index ? 'Nenhuma integração no catálogo' : 'Catálogo indisponível'}</h3><p>{entries.length ? 'Tente outro modelo ou fabricante.' : client.result.index ? 'O índice recebido não contém versões aprovadas. Os exemplos fictícios continuam somente no menu de desenvolvimento.' : 'Configure a URL ou tente atualizar. Os softwares locais continuam disponíveis.'}</p></div>}
    </>}
  </div>;
}
