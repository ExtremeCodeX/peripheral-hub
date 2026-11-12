import { memo, useId } from 'react';
import type { DemoMouse } from '../app/demo-mice';

// Ilustração vetorial original; sem imagens externas ou marcas de fabricantes.
export const MouseArtwork = memo(function MouseArtwork({ mouse }: { mouse: DemoMouse }) {
  const id = useId();
  const ergonomic = mouse.variant === 'ergo';
  const sport = mouse.variant === 'sport';
  const body = ergonomic
    ? 'M130 20C176 16 206 47 212 100L222 205C226 256 195 301 145 307C96 313 57 286 61 244L78 130C82 80 87 27 130 20Z'
    : sport
      ? 'M140 18C180 18 201 51 206 101L216 211C220 266 187 307 140 310C93 307 60 266 64 211L74 101C79 51 100 18 140 18Z'
      : 'M140 18C183 18 205 53 208 103L213 205C217 264 188 309 140 312C92 309 63 264 67 205L72 103C75 53 97 18 140 18Z';

  return (
    <svg className={`mouse-artwork mouse-artwork--${mouse.variant}`} viewBox="0 0 280 350" fill="none" aria-hidden="true" focusable="false">
      <defs>
        <linearGradient id={`${id}-shell`} x1="84" y1="40" x2="222" y2="256" gradientUnits="userSpaceOnUse">
          <stop stopColor={mouse.colors[0]} /><stop offset=".5" stopColor={mouse.colors[1]} /><stop offset="1" stopColor={mouse.colors[2]} />
        </linearGradient>
        <linearGradient id={`${id}-key`} x1="98" y1="35" x2="178" y2="162" gradientUnits="userSpaceOnUse">
          <stop stopColor={mouse.colors[0]} /><stop offset="1" stopColor={mouse.colors[1]} />
        </linearGradient>
        <linearGradient id={`${id}-edge`} x1="74" y1="140" x2="212" y2="140" gradientUnits="userSpaceOnUse">
          <stop stopColor="white" stopOpacity=".45" /><stop offset=".48" stopColor="white" stopOpacity=".04" /><stop offset="1" stopColor="black" stopOpacity=".3" />
        </linearGradient>
        <radialGradient id={`${id}-shadow`}>
          <stop stopColor="#000" stopOpacity=".55" /><stop offset="1" stopColor="#000" stopOpacity="0" />
        </radialGradient>
        <linearGradient id={`${id}-wheel`} x1="133" y1="68" x2="147" y2="68" gradientUnits="userSpaceOnUse">
          <stop stopColor="#1a1d20" /><stop offset=".5" stopColor="#505459" /><stop offset="1" stopColor="#17191c" />
        </linearGradient>
      </defs>
      <ellipse cx="146" cy="317" rx="101" ry="22" fill={`url(#${id}-shadow)`} />
      <path d={body} fill="#111416" transform="translate(0 4)" />
      <path d={body} fill={`url(#${id}-shell)`} stroke={`url(#${id}-edge)`} strokeWidth="2" />
      <path d={ergonomic ? 'M136 22C104 29 91 57 86 111L82 145L135 170Z' : 'M137 21C100 22 81 55 78 104L76 146L137 171Z'} fill={`url(#${id}-key)`} />
      <path d={ergonomic ? 'M144 22C180 22 201 52 206 101L211 143L144 169Z' : 'M143 21C180 22 199 55 202 104L204 146L143 171Z'} fill={`url(#${id}-key)`} />
      <path d={ergonomic ? 'M82 149L136 174M144 174L212 147M140 23V65' : 'M76 150L137 175M143 175L204 150M140 22V65'} stroke={mouse.colors[2]} strokeWidth="2" strokeLinecap="round" />
      <path d="M140 118v53" stroke={mouse.colors[2]} strokeWidth="2" />
      <rect x="130" y="65" width="20" height="54" rx="10" fill={mouse.colors[2]} />
      <rect x="133" y="68" width="14" height="46" rx="7" fill={`url(#${id}-wheel)`} />
      {[76, 82, 88, 94, 100, 106].map((y) => <path key={y} d={`M135 ${y}h10`} stroke="#858b8c" strokeOpacity=".55" />)}
      <rect x="136" y="133" width="8" height="16" rx="4" fill={mouse.accent} />
      <path d={ergonomic ? 'M76 138L72 170M70 182L67 209' : 'M70 135L68 164M67 175L65 200'} stroke={mouse.accent} strokeWidth="5" strokeLinecap="round" />
      <path d={ergonomic ? 'M202 176C211 230 194 274 162 288' : 'M200 180C205 233 184 280 155 292'} stroke="white" strokeOpacity=".12" strokeWidth="2" strokeLinecap="round" />
      <path d="M128 254h24m-18 6h12" stroke={mouse.accent} strokeWidth="2.5" strokeLinecap="round" opacity=".7" />
      {sport && <path d="m94 214 4 25m-10-8 4 18m95-35-4 25m10-8-4 18" stroke={mouse.colors[2]} strokeOpacity=".55" strokeWidth="2" strokeLinecap="round" />}
    </svg>
  );
});
