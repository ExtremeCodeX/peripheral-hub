import type { CSSProperties } from 'react';

type IconName = 'search' | 'menu' | 'close' | 'arrow' | 'check' | 'mouse' | 'chevron' | 'terminal' | 'back' | 'copy' | 'info' | 'partial' | 'review' | 'external';

const paths: Record<IconName, string> = {
  search: 'm21 21-4.5-4.5M19 10.5a8.5 8.5 0 1 1-17 0 8.5 8.5 0 0 1 17 0Z',
  menu: 'M4 6h16M4 12h16M4 18h10',
  close: 'm6 6 12 12M6 18 18 6',
  arrow: 'M5 12h14m-6-6 6 6-6 6',
  check: 'm5 12 4 4L19 6',
  mouse: 'M12 3v6m-6 0a6 6 0 0 1 12 0v6a6 6 0 0 1-12 0Z',
  chevron: 'm9 5 7 7-7 7',
  terminal: 'm4 6 5 5-5 5m8 1h7',
  back: 'M19 12H5m6-6-6 6 6 6',
  copy: 'M9 9h11v11H9ZM4 15H3V3h12v1',
  info: 'M12 11v6m0-10v.01M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0Z',
  partial: 'M7 12h10M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0Z',
  review: 'M8 19v-1a4 4 0 0 1 8 0v1M15 7a3 3 0 1 1-6 0 3 3 0 0 1 6 0Zm4 5a3 3 0 0 1 3 3v3M2 18v-3a3 3 0 0 1 3-3',
  external: 'M14 4h6v6m-1-5-9 9M10 4H4v16h16v-6',
};

export function Icon({ name, className, style }: { name: IconName; className?: string; style?: CSSProperties }) {
  return (
    <svg className={className} style={style} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d={paths[name]} />
    </svg>
  );
}
