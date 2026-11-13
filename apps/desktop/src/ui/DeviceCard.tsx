import type { DemoMouse } from '../app/demo-mice';
import { Icon } from './Icon';
import { MouseArtwork } from './MouseArtwork';
import { demoSoftware } from '../app/demo-software';
import { SoftwareStatus } from './SoftwareStatus';
import type { ButtonHTMLAttributes, ReactNode } from 'react';

export function DeviceCardFrame({ name, manufacturer, visual, status, actionLabel, children, ...button }: ButtonHTMLAttributes<HTMLButtonElement> & { name: string; manufacturer: string; visual?: ReactNode; status: ReactNode; actionLabel?: string; children?: ReactNode }) {
  return <button {...button} className="device-card" type="button"><span className="device-card__header"><span className="device-card__name">{name}</span><span className="device-card__manufacturer">{manufacturer}</span></span><span className="device-card__visual">{visual}</span><span className="device-card__footer">{status}<span className="device-card__action">{actionLabel}<span className="device-card__arrow" aria-hidden="true"><Icon name="arrow" /></span></span></span>{children}</button>;
}

export function DeviceCard({ mouse, onOpen }: { mouse: DemoMouse; onOpen: () => void }) {
  return (
    <DeviceCardFrame name={mouse.name} manufacturer={mouse.manufacturer} visual={<MouseArtwork mouse={mouse} />} status={<SoftwareStatus software={demoSoftware[mouse.id]} id={`status-${mouse.id}`} />} data-mouse-details={mouse.id} aria-label={`Ver detalhes de ${mouse.name}`} aria-describedby={`status-${mouse.id}`} onClick={onOpen} />
  );
}
