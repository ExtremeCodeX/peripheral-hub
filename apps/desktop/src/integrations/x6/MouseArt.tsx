/** Original X6 artwork, shared by workspace and bundled catalog/library cards. */
export function MouseArt() {
  return <svg viewBox="0 0 320 500" role="img" aria-label="Ilustração esquemática original do Attack Shark X6">
    <defs>
      <linearGradient id="x6-body" x1="0" x2="1"><stop stopColor="#101315"/><stop offset=".25" stopColor="#34383b"/><stop offset=".6" stopColor="#15191b"/><stop offset="1" stopColor="#3b4043"/></linearGradient>
      <linearGradient id="x6-top" x1="0" y1="0" x2=".8" y2="1"><stop stopColor="#33383c"/><stop offset=".7" stopColor="#111517"/></linearGradient>
      <radialGradient id="x6-glow"><stop stopColor="#354554" stopOpacity=".25"/><stop offset="1" stopColor="#0b0e10" stopOpacity="0"/></radialGradient>
    </defs>
    <ellipse cx="165" cy="310" rx="145" ry="178" fill="url(#x6-glow)"/>
    <path d="M148 52 C112 52 66 70 62 103 C56 154 68 211 63 268 C55 324 47 369 72 416 C106 481 211 485 250 427 C280 382 264 323 258 264 C253 210 265 155 256 107 C250 78 207 56 177 52 Z" fill="url(#x6-body)" stroke="#484e52" strokeWidth="1.2"/>
    <path d="M148 54 C112 54 69 72 65 103 L79 233 Q108 254 147 249 Z" fill="url(#x6-top)" stroke="#0a0d0f" strokeWidth="2"/>
    <path d="M177 54 C208 58 249 80 253 108 L244 233 Q215 256 176 249 Z" fill="url(#x6-top)" stroke="#090c0e" strokeWidth="2"/>
    <path d="M150 54 L150 249 Q164 260 176 249 L175 54" fill="#0b0e10" stroke="#3b4146"/>
    <rect x="151" y="99" width="24" height="57" rx="10" fill="#343b40" stroke="#7b858c"/>
    {Array.from({ length: 8 }, (_, i) => <path key={i} d={`M155 ${107 + i * 5} h16`} stroke="#111619" strokeWidth="2"/>)}
    <rect x="154" y="165" width="20" height="30" rx="9" fill="#222a30" stroke="#4b5862"/>
    <rect x="154" y="203" width="20" height="30" rx="9" fill="#222a30" stroke="#4b5862"/>
    <path d="M61 190 L64 236 Q54 235 54 226 L52 198 Q52 190 61 190 M63 252 L59 295 Q50 300 49 290 L52 260 Q53 252 63 252" fill="#29323a" stroke="#66727d"/>
    <path d="M106 376 Q160 408 222 370" fill="none" stroke="#626b71" strokeOpacity=".16"/>
    <text x="164" y="345" textAnchor="middle" fill="#68747d" fontSize="12" letterSpacing="5">X6</text>
  </svg>;
}

export function X6CardArt() { return <span className="x6-card-art"><MouseArt/></span>; }
