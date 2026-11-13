import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DeviceCard, DeviceCardFrame } from '../ui/DeviceCard';
import { WindowBar } from '../ui/WindowBar';
import { ThemeToggle } from '../ui/ThemeToggle';
import { CoverageBadge } from '../ui/SoftwareStatus';
import { Icon } from '../ui/Icon';
import { demoMice, filterMice, type DemoMouseId } from './demo-mice';
import { MouseDetails } from './MouseDetails';
import { demoExamples } from './demo-examples';
import { categoryLabels, filterSoftware, registeredSoftware, type LocalSoftware } from './local-software';
import { Catalog, useCatalog } from './Catalog';
import { starterPreviewIntegration } from '../integrations/starter/Preview';
import { x6Integration, publicX6Integration } from '../integrations/x6/Control';
import { publicReleases, type PublicRelease } from './distribution';

type Page = 'library' | 'explore' | 'examples';

export function App({ software = registeredSoftware }: { software?: readonly LocalSoftware[] }) {
  return <AppContent software={software} />;
}

function AppContent({ software }: { software: readonly LocalSoftware[] }) {
  const [page, setPage] = useState<Page>('library');
  const [starterRegistered, setStarterRegistered] = useState(false);
  const [x6Registered, setX6Registered] = useState(false);
  const [releases, setReleases] = useState<PublicRelease[]>(publicReleases);
  const [installing, setInstalling] = useState(false);
  const [installationError, setInstallationError] = useState('');
  useEffect(() => {
    let active = true;
    Promise.resolve(invoke<{ software: PublicRelease[] }>('software_distribution', { request: { action: 'list' } }))
      .then((result) => { if (active && Array.isArray(result?.software)) setReleases(result.software); })
      .catch(() => { if (active) setInstallationError('Não foi possível consultar as instalações públicas. Use o Hub desktop atualizado.'); });
    return () => { active = false; };
  }, []);
  const publicInstalled = releases.some((r) => r.id === publicX6Integration.id && r.installed);
  const localSoftware = [...software,
    ...(publicInstalled && !software.some((s) => s.id === publicX6Integration.id) ? [publicX6Integration] : []),
    ...(import.meta.env.DEV && starterRegistered && !software.some((s) => s.id === starterPreviewIntegration.id) ? [starterPreviewIntegration] : []),
     ...(import.meta.env.DEV && x6Registered && !publicInstalled && !software.some((s) => s.id === x6Integration.id) ? [x6Integration] : [])];
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
   const searchRef = useRef<HTMLInputElement>(null);
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
      (trigger ?? searchRef.current)?.focus({ preventScroll: true });
      window.scrollTo({ top: galleryScrollRef.current });
      localReturnRef.current = null;
    }
  }, [localView]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (document.querySelector('dialog[open]')) return;
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        searchRef.current?.focus();
      }
    }
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('keydown', onKeyDown);
    };
  }, []);

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
    window.scrollTo({ top: 0 });
  }

  function openLocal(entry: LocalSoftware) {
    if (!entry.available) return;
    galleryScrollRef.current = window.scrollY;
    returnFocusIdRef.current = null;
    localReturnRef.current = entry.id;
    setDetailsId(null);
    setLocalView(entry);
    window.scrollTo({ top: 0 });
  }

  async function installPublic(id: string) {
    if (installing) return;
    setInstalling(true); setInstallationError('');
    try {
      const result = await invoke<{ software: PublicRelease[] }>('software_distribution', { request: { action: 'install', plugin_id: id } });
      if (!Array.isArray(result?.software)) throw new Error('INVALID_RESULT');
      setReleases(result.software);
    } catch {
      setInstallationError('Instalação não concluída. Confira rede, versão do Hub, permissões e integridade do pacote. Nenhum código foi executado; tente novamente apenas após revisar o erro.');
    } finally { setInstalling(false); }
  }

  function openDetails(id: DemoMouseId) {
    galleryScrollRef.current = window.scrollY;
    returnFocusIdRef.current = id;
    setDetailsId(id);
    window.scrollTo({ top: 0 });
  }

  return (
    <><WindowBar /><div className={`app-shell${localView?.workspace ? ' app-shell--workspace' : ''}`}>
      <a className="skip-link" href="#devices">Ir para o conteúdo</a>
      <div hidden={!!localView?.workspace}>
      <header className="app-header">
        <h1 className="wordmark">Peripheral <span>Hub</span></h1>
        <div className="header-tools">
          <div className="search-field" role="search">
            <Icon name="search" />
            <input ref={searchRef} type="search" aria-label={page === 'library' ? 'Pesquisar meus softwares' : page === 'explore' ? 'Pesquisar catálogo' : 'Pesquisar softwares fictícios'} placeholder={page === 'library' ? 'Pesquisar meus softwares' : page === 'explore' ? 'Pesquisar catálogo' : 'Pesquisar softwares fictícios'} value={query} onChange={(event) => { setQuery(event.target.value); returnFocusIdRef.current = null; localReturnRef.current = null; setDetailsId(null); setLocalView(null); }} onKeyDown={(event) => { if (event.key === 'Escape') clearSearch(); }} />
            {query ? <button className="search-clear" type="button" aria-label="Limpar pesquisa" onClick={clearSearch}><Icon name="close" /></button> : <kbd className="search-shortcut" aria-hidden="true">Ctrl K</kbd>}
          </div>
          <ThemeToggle />
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
          <div className="collection-heading">
              <div><h2 id="collection-title" tabIndex={-1}>{examplesOpen ? 'Exemplos para desenvolvedores' : page === 'library' ? 'Meus softwares' : 'Explorar softwares'}</h2>{examplesOpen && <p>Casos de referência para integrações. Todos os dados são fictícios.</p>}</div>
            {examplesOpen && <span className="demo-label"><span aria-hidden="true" />Modo demonstração</span>}
          </div>
          {page === 'library' ? <>
            <div className="collection-toolbar"><span className="collection-category"><Icon name="mouse" />{visibleSoftware.some((entry) => entry.category && entry.category !== 'mouse') ? 'Softwares locais' : 'Mouses'}<span className="count-badge">{visibleSoftware.length}</span></span></div>
            {visibleSoftware.length ? <div className="device-grid" aria-label="Softwares registrados">{visibleSoftware.map((entry) => {
              const Visual = entry.Visual;
              const Presence = entry.Presence;
              return <DeviceCardFrame key={entry.id} name={entry.name} manufacturer={`${entry.manufacturer}${entry.category && entry.showCategoryOnCard !== false ? ` · ${categoryLabels[entry.category]}` : ''}`} data-software={entry.id} disabled={!entry.available} onClick={() => openLocal(entry)} aria-label={`Abrir ${entry.name}`} actionLabel={entry.available ? 'Abrir controles' : undefined} visual={Visual && <Visual />} status={entry.available && entry.coverage ? <CoverageBadge coverage={entry.coverage} label={entry.status} title="Suporte no escopo documentado da integração. Consulte os limites nos detalhes." /> : <span className="software-card__status">{entry.available ? entry.status : 'Indisponível · abertura desativada'}</span>}>{Presence && <Presence />}</DeviceCardFrame>;
            })}</div> :
              <div className="empty-state"><Icon name={localSoftware.length ? 'search' : 'mouse'} /><h3>{localSoftware.length ? 'Nenhum software encontrado.' : 'Sua biblioteca está vazia'}</h3>{localSoftware.length ? <><p>Nenhum software registrado corresponde a “{query}”.</p><button className="text-button" type="button" onClick={clearSearch}>Limpar pesquisa<Icon name="arrow" /></button></> : <><p>Encontre a integração do seu mouse em Explorar softwares. Depois de instalada, ela aparece aqui.</p><button className="text-button" type="button" onClick={() => navigate('explore')}>Explorar softwares<Icon name="arrow" /></button></>}</div>}
          </> : page === 'explore' ? <Catalog client={catalog} query={query} releases={releases} installing={installing} installationError={installationError} onInstall={installPublic} onOpen={() => openLocal(publicX6Integration)} /> : <>
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
      {import.meta.env.DEV && !LocalView && <details className="dev-tools"><summary>Ferramentas de desenvolvimento</summary><div>
        <button className="menu-action" type="button" onClick={() => navigate('examples')}>Exemplos para desenvolvedores</button>
        <button className="menu-action" type="button" disabled={starterRegistered} onClick={() => { setStarterRegistered(true); setQueries((current) => ({ ...current, library: '' })); navigate('library'); }}>{starterRegistered ? 'Starter local registrado (DEV)' : 'Registrar starter local (DEV)'}</button>
        <button className="menu-action" type="button" disabled={x6Registered} onClick={() => { setX6Registered(true); setQueries((current) => ({ ...current, library: '' })); navigate('library'); }}>{x6Registered ? 'X6 Control registrado (DEV)' : 'Registrar X6 Control (DEV)'}</button>
      </div></details>}
    </div></>
  );
}
