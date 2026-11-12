import { useEffect, useRef, useState } from 'react';
import type { DemoMouse } from '../app/demo-mice';
import type { DemoSoftware } from '../app/demo-software';
import { buildReviewPrompt } from '../app/review-prompt';
import { copyText } from './copy-text';
import { Icon } from './Icon';

export function ReviewDialog({ mouse, software, colorLabel, onClose }: { mouse: DemoMouse; software: DemoSoftware; colorLabel?: string; onClose: () => void }) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const closeRef = useRef<HTMLButtonElement>(null);
  const bodyRef = useRef<HTMLDivElement>(null);
  const [showPrompt, setShowPrompt] = useState(false);
  const [copyState, setCopyState] = useState<'idle' | 'copying' | 'copied' | 'error'>('idle');
  const prompt = buildReviewPrompt(mouse, software, colorLabel);

  useEffect(() => {
    const dialog = dialogRef.current!;
    const opener = document.activeElement;
    dialog.showModal();
    closeRef.current?.focus();
    const oldOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    return () => {
      if (dialog.open) dialog.close();
      document.body.style.overflow = oldOverflow;
      if (opener instanceof HTMLElement && opener.isConnected) opener.focus({ preventScroll: true });
    };
  }, []);

  async function copyPrompt() {
    setCopyState('copying');
    try {
      await copyText(prompt);
      setCopyState('copied');
    } catch {
      setCopyState('error');
    }
  }

  function changeView() {
    setShowPrompt(!showPrompt);
    if (bodyRef.current) bodyRef.current.scrollTop = 0;
  }

  return (
    <dialog ref={dialogRef} className="review-dialog" aria-labelledby="review-title" aria-describedby="review-description" onCancel={(event) => { event.preventDefault(); onClose(); }} onClick={(event) => { if (event.target === event.currentTarget) onClose(); }}>
      <div className="review-dialog__content">
        <header className="review-dialog__header">
          <div><h2 id="review-title">{showPrompt ? 'Prompt de avaliação' : 'Sua experiência ajuda a validar.'}</h2><p id="review-description">{mouse.name}{colorLabel && ` · ${colorLabel}`} · {software.version} · Exemplo fictício</p></div>
          <button ref={closeRef} className="icon-button" type="button" aria-label="Fechar avaliação" onClick={onClose}><Icon name="close" /></button>
        </header>
        <div className="review-dialog__body" ref={bodyRef} tabIndex={0} aria-label={showPrompt ? 'Leitura do prompt completo' : 'Instruções de avaliação'}>
        {showPrompt ? (
          <div className="prompt-view"><p>Revise o conteúdo antes de compartilhar. O prompt não é executado nem enviado pelo Hub.</p><pre tabIndex={0} aria-label="Conteúdo do prompt de avaliação">{prompt}</pre></div>
        ) : (
          <>
            <p className="review-intro">Relate o que você realmente testou. O objetivo é ajudar outros usuários e o mantenedor a entender funções, falhas e limites.</p>
            <ol className="review-steps">
              <li><strong>Teste e reúna evidências.</strong><p>Na integração real, use seu periférico físico e grave um vídeo mostrando o dispositivo e as funções testadas. Marque o que não foi testado.</p></li>
              <li><strong>Organize seu relato com o prompt.</strong><p>Copie o roteiro abaixo para um chat de IA de sua escolha. Ele fará perguntas e, após sua confirmação, preparará um relatório Markdown. Se o chat permitir, poderá oferecer o arquivo .md.</p></li>
              <li><strong>Revise e publique no repositório certo.</strong><p>Abra o repositório da integração e siga o CONTRIBUTING e o template indicado para Issues ou Discussions. Inclua versão, ambiente, resultados e links das evidências. Remova dados pessoais e segredos.</p></li>
            </ol>
            <div className="review-notice"><Icon name="info" /><p><strong>Nesta demonstração não há repositório vinculado.</strong> Nenhuma avaliação será publicada ou aprovação registrada. No processo real, um relato não remove BETA automaticamente: são necessárias duas aprovações humanas e a decisão do mantenedor.</p></div>
            <p className="review-safety">Evidência funcional não é auditoria de segurança. Vídeo e aprovações não garantem ausência de vulnerabilidades ou bugs. O Hub não envia dados a uma IA.</p>
          </>
        )}
        </div>
        <footer className="review-dialog__footer">
          <div className="review-dialog__actions">
            <button className="primary-button" type="button" disabled={copyState === 'copying'} onClick={copyPrompt}><Icon name={copyState === 'copied' ? 'check' : 'copy'} />{copyState === 'copying' ? 'Copiando…' : copyState === 'copied' ? 'Copiar novamente' : 'Copiar prompt de avaliação'}</button>
            <button className="secondary-button" type="button" aria-expanded={showPrompt} onClick={changeView}><Icon name={showPrompt ? 'back' : 'chevron'} />{showPrompt ? 'Voltar às instruções' : 'Ver prompt'}</button>
          </div>
          <p className={`copy-feedback${copyState === 'error' ? ' copy-feedback--error' : ''}`} role="status">{copyState === 'copied' ? 'Prompt copiado. Cole no chat de IA que você escolher.' : copyState === 'error' ? showPrompt ? 'Não foi possível copiar. Selecione o texto exibido acima e copie manualmente.' : 'Não foi possível copiar. Abra “Ver prompt” e copie o texto manualmente.' : 'Você controla o que compartilha. Nenhum envio automático.'}</p>
        </footer>
      </div>
    </dialog>
  );
}
