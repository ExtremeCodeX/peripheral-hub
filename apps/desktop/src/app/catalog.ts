import { invoke } from '@tauri-apps/api/core';

export const CATALOG_INTERVAL = 15 * 60 * 1000;
export type CatalogSoftware = {
  id: string; name: string; version: string; shortDescription: string; fullDescription: string;
  authors: string[]; license: string; architectures: string[];
  devices: { model: string; manufacturer: string; category: string; compatibility: string }[];
  features: { name: string; implementation: string; verification: string; notes: string }[];
  requirements: string[]; limitations: string[];
  source: { repository: string; commit: string };
};
export type CatalogResult = {
  index: { schemaVersion: 1; revision: string; software: CatalogSoftware[] } | null;
  status: string; cacheWarning: string | null; checkedAt: number | null;
};
export const catalogMessages: Record<string, string> = {
  NOT_CONFIGURED: 'Catálogo não configurado. Consulte docs/public/CATALOG.md para definir a URL no build.',
  CONFIGURATION: 'URL do catálogo inválida. Revise a configuração do build.',
  CACHED: 'Última versão válida salva neste computador.',
  UPDATED: 'Catálogo atualizado.', NOT_MODIFIED: 'O catálogo já está atualizado.',
  NETWORK: 'Sem conexão com o catálogo. Tente atualizar novamente.',
  TIMEOUT: 'O catálogo demorou para responder. Tente novamente.',
  HTTP_ERROR: 'O servidor não disponibilizou o catálogo. Tente novamente mais tarde.',
  REDIRECT: 'Redirecionamento recusado. Revise a URL configurada.',
  CONTENT_TYPE: 'O servidor não retornou JSON sem compressão.',
  TOO_LARGE: 'O catálogo excedeu o limite de 2 MiB.',
  UNSUPPORTED_VERSION: 'Versão do catálogo incompatível com este Hub.',
  INVALID_JSON: 'O catálogo recebido não contém JSON válido.',
  INVALID_CONTENT: 'O catálogo recebido tem metadados inválidos.',
  INVALID_REVISION: 'A revisão do catálogo não corresponde ao conteúdo.',
  UNSAFE_ADDRESS: 'O endereço do catálogo não é público e foi bloqueado.',
  UNEXPECTED_304: 'O servidor retornou uma versão em cache que não existe neste computador.',
  PYTHON_UNAVAILABLE: 'Python 3 não está disponível para consultar o catálogo.',
  WORKER_ERROR: 'Não foi possível consultar o catálogo. Tente novamente.',
  BUSY: 'Uma consulta de catálogo já está em andamento. Tente novamente.',
  CACHE_ERROR: 'Não foi possível acessar o diretório do cache local.',
};

export class CatalogClient {
  result: CatalogResult = { index: null, status: 'NOT_CONFIGURED', cacheWarning: null, checkedAt: null };
  busy = false;
  private timer: ReturnType<typeof setInterval> | undefined;
  private pending: Promise<void> | undefined;
  private generation = 0;
  constructor(private notify: () => void, private call = (refresh: boolean) => invoke<CatalogResult>('catalog_sync', { refresh })) {}
  enter() {
    if (!this.timer) this.timer = setInterval(() => { void this.refresh(); }, CATALOG_INTERVAL);
    return this.refresh();
  }
  refresh() {
    if (this.pending) return this.pending;
    const generation = this.generation;
    this.busy = true;
    this.notify();
    const accept = (next: CatalogResult) => {
      if (generation !== this.generation) return;
      this.result = { ...next, index: next.index ?? this.result.index };
      this.notify();
    };
    this.pending = (async () => {
      try {
        if (!this.result.index) accept(await this.call(false));
        if (generation !== this.generation) return;
        accept(await this.call(true));
      } catch (error) {
        if (generation === this.generation) this.result = { ...this.result, status: typeof error === 'string' && error in catalogMessages ? error : 'WORKER_ERROR' };
      } finally {
        if (generation === this.generation) { this.busy = false; this.pending = undefined; this.notify(); }
      }
    })();
    return this.pending;
  }
  dispose() { clearInterval(this.timer); this.timer = undefined; this.generation++; this.pending = undefined; this.busy = false; }
}
