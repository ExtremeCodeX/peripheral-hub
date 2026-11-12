import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DeviceCard, DeviceCardFrame } from '../ui/DeviceCard';
import { WindowBar } from '../ui/WindowBar';
import { CoverageBadge } from '../ui/SoftwareStatus';
import { Icon } from '../ui/Icon';
import { demoMice, filterMice, type DemoMouseId } from './demo-mice';
import { MouseDetails } from './MouseDetails';
import { demoExamples } from './demo-examples';
import { categoryLabels, filterSoftware, registeredSoftware, type LocalSoftware } from './local-software';
import { Catalog, useCatalog } from './Catalog';
import { starterPreviewIntegration } from '../integrations/starter/Preview';

type Page = 'library' | 'explore' | 'examples';

export function App({ software = registeredSoftware }: { software?: readonly LocalSoftware[] }) {
  return <AppContent software={software} />;
}

function AppContent({ software }: { software: readonly LocalSoftware[] }) {
  const [page, setPage] = useState<Page>('library');
  const [starterRegistered, setStarterRegistered] = useState(false);
  const localSoftware = import.meta.env.DEV && starterRegistered
    ? [...software.filter((entry) => entry.id !== starterPreviewIntegration.id), starterPreviewIntegration] : software;
  const catalog = useCatalog(page === 'explore');
  const [queries, setQueries] = useState<Record<Page, string>>({ library: '', explore: '', examples: '' });
  const query = queries[page];
  const setQuery = (value: string) => setQueries((current) => ({ ...current, [page]: value }));
  const examplesOpen = page === 'examples';
  const [detailsId, setDetailsId] = useState<DemoMouseId | null>(null);
  const [localView, setLocalView] = useState<LocalSoftware | null>(null);
  const localReturnRef = useRef<string | null>(null);
  const galleryScrollRef = useRef(0);
  const returnFocusIdRef = useRef<DemoMouseId | null>(null);
  const focusCollectionRef = useRef(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [checking, setChecking] = useState(false);
  const [backendStatus, setBackendStatus] = useState('Verifique a comunicação com o aplicativo desktop.');
  const searchRef = useRef<HTMLInputElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const menuButtonRef = useRef<HTMLButtonElement>(null);
  const firstMenuActionRef = useRef<HTMLButtonElement>(null);
  const visibleMice = filterMice(query);
  const activeMouse = demoMice.find((mouse) => mouse.id === detailsId);
  const visibleSoftware = filterSoftware(localSoftware, query);
  const LocalView = localView?.View;

  useEffect(() => {
    if (focusCollectionRef.current && !detailsId && !localView) {
      document.getElementById('collection-title')?.focus();
      focusCollectionRef.current = false;
    }
  });

  useEffect(() => {
    if (!detailsId && returnFocusIdRef.current) {
      document.querySelector<HTMLButtonElement>(`[data-mouse-details="${returnFocusIdRef.current}"]`)?.focus({ preventScroll: true });
      window.scrollTo({ top: galleryScrollRef.current });
      returnFocusIdRef.current = null;
    }
  }, [detailsId]);

  useEffect(() => {
    if (!localView && localReturnRef.current) {
      const trigger = Array.from(document.querySelectorAll<HTMLButtonElement>('[data-software]')).find((button) => button.dataset.software === localReturnRef.current);
      (trigger ?? menuButtonRef.current)?.focus({ preventScroll: true });
      window.scrollTo({ top: galleryScrollRef.current });
      localReturnRef.current = null;
    }
  }, [localView]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (document.querySelector('dialog[open]')) return;
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setMenuOpen(false);
        searchRef.current?.focus();
      }
      if (event.key === 'Escape' && menuOpen) {
        setMenuOpen(false);
        menuButtonRef.current?.focus();
      }
    }
    function onPointerDown(event: PointerEvent) {
      if (event.target instanceof Node && !menuRef.current?.contains(event.target)) setMenuOpen(false);
    }
    document.addEventListener('keydown', onKeyDown);
    document.addEventListener('pointerdown', onPointerDown);
    return () => {
      document.removeEventListener('keydown', onKeyDown);
      document.removeEventListener('pointerdown', onPointerDown);
    };
  }, [menuOpen]);

  useEffect(() => {
    if (menuOpen) firstMenuActionRef.current?.focus();
  }, [menuOpen]);

  async function checkBackend() {
    setChecking(true);
    setBackendStatus('Verificando comunicação com o backend…');
    try {
      setBackendStatus(await invoke<string>('check_backend'));
    } catch {
      setBackendStatus('Não foi possível acessar o backend. Abra o aplicativo desktop e tente novamente.');
    } finally {
      setChecking(false);
    }
  }

  function clearSearch() {
    setQuery('');
    returnFocusIdRef.current = null;
    localReturnRef.current = null;
    setDetailsId(null);
    setLocalView(null);
    searchRef.current?.focus();
  }

  function navigate(next: Page) {
    setPage(next);
    setDetailsId(null);
    setLocalView(null);
    returnFocusIdRef.current = null;
    localReturnRef.current = null;
    focusCollectionRef.current = true;
    setMenuOpen(false);
    window.scrollTo({ top: 0 });
  }

  function openLocal(entry: LocalSoftware) {
    if (!entry.available) return;
    galleryScrollRef.current = window.scrollY;
    returnFocusIdRef.current = null;
    localReturnRef.current = entry.id;
    setDetailsId(null);
    setLocalView(entry);
    setMenuOpen(false);
    window.scrollTo({ top: 0 });
  }

  function openDetails(id: DemoMouseId) {
    galleryScrollRef.current = window.scrollY;
    returnFocusIdRef.current = id;
    setDetailsId(id);
    setMenuOpen(false);
    window.scrollTo({ top: 0 });
  }

  return (
    <><WindowBar /><div className={`app-shell${localView?.workspace ? ' app-shell--workspace' : ''}`}>
      <a className="skip-link" href="#devices">Ir para o conteúdo</a>
      <div hidden={!!localView?.workspace}>
      <header className="app-header">
        <h1 className="wordmark">Peripheral <span>Hub</span><span className="wordmark__dot" aria-hidden="true" /></h1>
        <div className="header-tools">
          <div className="search-field" role="search">
            <Icon name="search" />
            <input ref={searchRef} type="search" aria-label={page === 'library' ? 'Pesquisar meus softwares' : page === 'explore' ? 'Pesquisar catálogo' : 'Pesquisar softwares fictícios'} placeholder={page === 'library' ? 'Pesquisar meus softwares' : page === 'explore' ? 'Pesquisar catálogo' : 'Pesquisar softwares fictícios'} value={query} onChange={(event) => { setQuery(event.target.value); returnFocusIdRef.current = null; localReturnRef.current = null; setDetailsId(null); setLocalView(null); }} onKeyDown={(event) => { if (event.key === 'Escape') clearSearch(); }} />
            {query ? <button className="search-clear" type="button" aria-label="Limpar pesquisa" onClick={clearSearch}><Icon name="close" /></button> : <kbd className="search-shortcut" aria-hidden="true">Ctrl K</kbd>}
          </div>
          <div className="app-menu" ref={menuRef} onBlur={(event) => { if (!event.currentTarget.contains(event.relatedTarget)) setMenuOpen(false); }}>
            <button ref={menuButtonRef} className="icon-button menu-toggle" type="button" aria-label="Menu do aplicativo" aria-expanded={menuOpen} aria-controls="app-menu-panel" onClick={() => setMenuOpen(!menuOpen)}><Icon name={menuOpen ? 'close' : 'menu'} /></button>
            {menuOpen && (
              <div id="app-menu-panel" className="menu-panel" role="region" aria-label="Opções do aplicativo">
                <span className="menu-panel__title">Sua área de trabalho</span>
                 <button ref={firstMenuActionRef} className="menu-action" type="button" onClick={() => navigate('library')}><Icon name="mouse" />Meus softwares<Icon name="chevron" /></button>
                 <button className="menu-action" type="button" onClick={() => navigate('explore')}><Icon name="search" />Explorar softwares<Icon name="chevron" /></button>
                  {import.meta.env.DEV && <button className="menu-action" type="button" onClick={() => navigate('examples')}><Icon name="terminal" />Exemplos para desenvolvedores<Icon name="chevron" /></button>}
                  {import.meta.env.DEV && <button className="menu-action" type="button" disabled={starterRegistered} onClick={() => { setStarterRegistered(true); setQueries((current) => ({ ...current, library: '' })); navigate('library'); }}><Icon name="terminal" />{starterRegistered ? 'Starter local registrado (DEV)' : 'Registrar starter local (DEV)'}<Icon name="chevron" /></button>}
                <div className="menu-diagnostic">
                  <button className="menu-action" type="button" disabled={checking} onClick={checkBackend}><Icon name="terminal" />{checking ? 'Verificando…' : 'Verificar backend'}</button>
                  <p role="status">{backendStatus}</p>
                </div>
              </div>
            )}
          </div>
        </div>
      </header>

      <nav className="page-navigation" aria-label="Navegação principal">
        <button type="button" aria-current={page === 'library' && !LocalView ? 'page' : undefined} onClick={() => navigate('library')}>Meus softwares</button>
        <button type="button" aria-current={page === 'explore' && !LocalView ? 'page' : undefined} onClick={() => navigate('explore')}>Explorar softwares</button>
      </nav>
      </div>

      <main id="devices" tabIndex={-1}>
        {LocalView ? <LocalView onBack={() => setLocalView(null)} /> : activeMouse ? <MouseDetails key={activeMouse.id} mouse={activeMouse} fromExamples={examplesOpen} onBack={() => setDetailsId(null)} /> :
        <section className="collection" aria-labelledby="collection-title">
          {page === 'library' && <p>R1: os controles ficam no aplicativo local R1 Control separado. Este núcleo não configura o R1 sozinho.</p>}
          <div className="collection-heading">
             <div><h2 id="collection-title" tabIndex={-1}>{examplesOpen ? 'Exemplos para desenvolvedores' : page === 'library' ? 'Meus softwares' : 'Explorar softwares'}</h2>{page !== 'library' && <p>{examplesOpen ? 'Casos de referência para integrações. Todos os dados são fictícios.' : 'Versões do catálogo curado. Somente consulta; sem instalação ou execução.'}</p>}</div>
            {examplesOpen && <span className="demo-label"><span aria-hidden="true" />Modo demonstração</span>}
          </div>
          {page === 'library' ? <>
            <div className="collection-toolbar"><span className="collection-category"><Icon name="mouse" />{visibleSoftware.some((entry) => entry.category && entry.category !== 'mouse') ? 'Softwares locais' : 'Mouses'}<span className="count-badge">{visibleSoftware.length}</span></span></div>
            {visibleSoftware.length ? <div className="device-grid" aria-label="Softwares registrados">{visibleSoftware.map((entry) => {
              const Visual = entry.Visual;
              const Presence = entry.Presence;
              return <DeviceCardFrame key={entry.id} name={entry.name} manufacturer={`${entry.manufacturer}${entry.category && entry.showCategoryOnCard !== false ? ` · ${categoryLabels[entry.category]}` : ''}`} data-software={entry.id} disabled={!entry.available} onClick={() => openLocal(entry)} aria-label={`Abrir ${entry.name}`} visual={Visual && <Visual />} status={entry.available && entry.coverage ? <CoverageBadge coverage={entry.coverage} label={entry.status} title="Suporte no escopo documentado da integração. Consulte os limites nos detalhes." /> : <span className="software-card__status">{entry.available ? entry.status : 'Indisponível · abertura desativada'}</span>}>{Presence && <Presence />}</DeviceCardFrame>;
            })}</div> :
              <div className="empty-state"><Icon name={software.length ? 'search' : 'mouse'} /><h3>{software.length ? 'Nenhum software encontrado.' : 'Sua biblioteca está vazia'}</h3>{software.length ? <><p>Nenhum software registrado corresponde a “{query}”.</p><button className="text-button" type="button" onClick={clearSearch}>Limpar pesquisa<Icon name="arrow" /></button></> : <><p>Você ainda não adicionou nenhum software ao Peripheral Hub. Explore os softwares disponíveis e comece a configurar seus periféricos.</p><button className="text-button" type="button" onClick={() => navigate('explore')}>Explorar softwares<Icon name="arrow" /></button></>}</div>}
          </> : page === 'explore' ? <Catalog client={catalog} query={query} /> : <>
          <div className="collection-toolbar">
            <span className="collection-category"><Icon name="mouse" />Mouses<span className="count-badge">{visibleMice.length}</span></span>
            <span className="collection-hint">Modelos fictícios · Sem conexão com hardware</span>
          </div>
          {visibleMice.length ? (
            <div className="device-grid" aria-label="Mouses de demonstração">
              {visibleMice.map((mouse) => <div className="gallery-item" key={mouse.id}><DeviceCard mouse={mouse} onOpen={() => openDetails(mouse.id)} />{examplesOpen && <div className="example-case"><h3>{demoExamples[mouse.id].title}</h3><p>{demoExamples[mouse.id].description}</p></div>}</div>)}
            </div>
          ) : (
            <div className="empty-state"><Icon name="search" /><h3>Nenhum mouse por aqui.</h3><p>Nenhum exemplo corresponde a “{query}”. Tente outro modelo ou fabricante.</p><button className="text-button" type="button" onClick={clearSearch}>Ver todos os exemplos<Icon name="arrow" /></button></div>
          )}
          <footer className="collection-footer">
            <span className="result-count" aria-live="polite">{visibleMice.length} de {demoMice.length} exemplos</span>
          </footer>
          </>}
        </section>
        }
      </main>
    </div></>
  );
}
