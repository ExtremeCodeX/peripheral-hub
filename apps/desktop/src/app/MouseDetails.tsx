import { useEffect, useRef, useState } from 'react';
import type { DemoMouse } from './demo-mice';
import { demoColorCases, demoSoftware } from './demo-software';
import { Icon } from '../ui/Icon';
import { MouseArtwork } from '../ui/MouseArtwork';
import { SoftwareStatus } from '../ui/SoftwareStatus';
import { ReviewDialog } from '../ui/ReviewDialog';
import { ExternalLink } from '../ui/ExternalLink';

export function MouseDetails({ mouse, onBack, fromExamples = false }: { mouse: DemoMouse; onBack: () => void; fromExamples?: boolean }) {
  const colorCase = demoColorCases[mouse.id];
  const [colorId, setColorId] = useState(colorCase?.options[0]?.id);
  const colorOption = colorCase?.options.find((option) => option.id === colorId);
  const software = colorOption?.software ?? demoSoftware[mouse.id];
  const artwork = colorOption ? { ...mouse, colors: colorOption.colors, accent: colorOption.accent } : mouse;
  const [reviewOpen, setReviewOpen] = useState(false);
  const headingRef = useRef<HTMLHeadingElement>(null);
  const beta = software.validation === 'beta';

  useEffect(() => { headingRef.current?.focus({ preventScroll: true }); }, [mouse.id]);

  return (
    <section className="mouse-details" aria-labelledby="mouse-details-title">
      <div className="details-navigation"><button className="back-button" type="button" onClick={onBack}><Icon name="back" />{fromExamples ? 'Voltar aos exemplos' : 'Voltar à exploração'}</button><span className="demo-label"><span aria-hidden="true" />Dados fictícios</span></div>
      <div className="details-hero">
        <div className="details-artwork"><MouseArtwork mouse={artwork} /></div>
        <div className="details-overview">
          <h2 id="mouse-details-title" ref={headingRef} tabIndex={-1}>{mouse.name}</h2>
          <p className="details-manufacturer">{mouse.manufacturer}</p>
          <dl className="software-facts"><div><dt>Criador</dt><dd><ExternalLink key={`creator-${software.version}`} label={software.creator.name} url={software.creator.profileUrl} kind="perfil" /></dd></div><div><dt>Versão</dt><dd><ExternalLink key={`version-${software.version}`} label={software.version} url={software.repositoryUrl} kind="repositório" /></dd></div></dl>
          <SoftwareStatus software={software} onBetaClick={() => setReviewOpen(true)} />
          {colorCase && <fieldset className="color-variants"><legend>Cor / variante do hardware</legend><p>{colorCase.reason}</p><div className="color-variants__options">{colorCase.options.map((option) => <button key={option.id} type="button" className="color-option" aria-pressed={colorId === option.id} onClick={() => setColorId(option.id)}><span className="variant-swatch" style={{ backgroundColor: option.colors[1] }} aria-hidden="true" />{option.label}</button>)}</div><p className="variant-version" role="status">{colorOption?.label} · {software.version}</p></fieldset>}
        </div>
      </div>
      <div className="details-body">
        <section className="feature-section" aria-labelledby="features-title">
          <div className="section-heading"><h3 id="features-title">Funções</h3></div>
          <ul className="feature-list">{software.features.map((feature) => <li key={feature.name}><span className={`feature-icon${feature.available ? ' feature-icon--available' : ''}`}><Icon name={feature.available ? 'check' : 'partial'} /></span><div><strong>{feature.name}</strong><p>{feature.note}</p></div><span className={`feature-state${feature.available ? ' feature-state--available' : ''}`}>{feature.available ? 'Disponível' : 'Não implementado'}</span></li>)}</ul>
        </section>
        <aside className="validation-panel" aria-labelledby="validation-title">
          <div className="validation-panel__heading"><Icon name="review" /><h3 id="validation-title">Validação da comunidade</h3></div>
          {beta ? <>
            <p>Vídeo aceito · cenário fictício</p>
            <div className="approval-summary"><span className="approval-steps" aria-hidden="true">{[1, 2].map((step) => <span key={step} className={software.approvals >= step ? 'approval-step approval-step--done' : 'approval-step'}><Icon name={software.approvals >= step ? 'check' : 'review'} /></span>)}</span><div><strong>{software.approvals} de 2 aprovações</strong><span>{software.approvals === 1 ? 'Falta 1 aprovação humana' : 'Faltam 2 aprovações humanas'}</span></div></div>
            <button className="primary-button" type="button" onClick={(event) => { event.currentTarget.focus({ preventScroll: true }); setReviewOpen(true); }}>Avalie esta integração<Icon name="arrow" /></button>
            <p className="validation-footnote">Decisão final do mantenedor.</p>
          </> : <div className="reviewed-message"><Icon name="check" /><span>Revisada no exemplo</span></div>}
        </aside>
      </div>
      {reviewOpen && <ReviewDialog mouse={mouse} software={software} colorLabel={colorOption?.label} onClose={() => setReviewOpen(false)} />}
    </section>
  );
}
