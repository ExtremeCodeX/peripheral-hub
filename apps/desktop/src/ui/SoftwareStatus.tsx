import type { DemoSoftware } from '../app/demo-software';
import { coverageLabel } from '../app/demo-software';
import { Icon } from './Icon';

export function CoverageBadge({ coverage, label, title }: { coverage: 'complete' | 'partial'; label: string; title: string }) {
  return <span className={`coverage-badge coverage-badge--${coverage}`} title={title}><Icon name={coverage === 'complete' ? 'check' : 'partial'} />{label}</span>;
}

export function SoftwareStatus({ software, id, onBetaClick }: { software: DemoSoftware; id?: string; onBetaClick?: () => void }) {
  return (
    <span className="software-status" id={id}>
      <CoverageBadge coverage={software.coverage} label={coverageLabel(software)} title={software.coverage === 'complete' ? 'Todas as funções do escopo documentado. Estado fictício nesta prévia.' : 'Apenas parte das funções do escopo documentado. Estado fictício nesta prévia.'} />
      {software.validation === 'beta' && (onBetaClick ? <button className="beta-badge" type="button" aria-label="Entender a validação BETA" onClick={(event) => { event.currentTarget.focus({ preventScroll: true }); onBetaClick(); }}>BETA</button> : <span className="beta-badge" title="Vídeo aceito no cenário de exemplo; são necessárias duas aprovações humanas. Não há avaliações reais.">BETA</span>)}
    </span>
  );
}
