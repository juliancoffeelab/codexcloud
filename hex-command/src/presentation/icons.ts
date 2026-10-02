// Small outline symbols share the controls' stroke style and never depend on fonts.
const paths = {
  move: '<path d="m5 17 4-4 3 3 7-9M14 7h5v5"/>',
  blade: '<path d="m5 19 3-3m-2-3 5 5m-3-4L17 4l3-1-1 3-10 9"/>',
  pierce: '<path d="m5 19 14-14M13 5h6v6M5 14v5h5"/>',
  impact: '<path d="m5 19 10-10M11 5l4-3 7 7-3 4z"/>',
  range:
    '<circle cx="12" cy="12" r="7"/><circle cx="12" cy="12" r="3"/><path d="M12 2v3m0 14v3M2 12h3m14 0h3"/>',
  shield: '<path d="M12 3 4 6v6c0 4 4 7 8 9 4-2 8-5 8-9V6z"/>',
  blocked: '<path d="M3 12h18m-4-4 4 4-4 4M8 4v16m8-16L8 20"/>',
  coin: '<circle cx="12" cy="12" r="8"/><path d="M14 8h-3a2 2 0 0 0 0 4h2a2 2 0 0 1 0 4h-3m2-10v12"/>',
  heal: '<path d="M9 3h6v6h6v6h-6v6H9v-6H3V9h6z"/>',
  recruit: '<circle cx="9" cy="7" r="3"/><path d="M3 21v-3a6 6 0 0 1 12 0v3m4-12v8m-4-4h8"/>',
  keep: '<path d="M4 21V3h4v4h3V3h3v4h3V3h3v18zM10 21v-6h4v6"/>',
  search: '<circle cx="10" cy="10" r="6"/><path d="m15 15 6 6"/>',
};

export type IconName = keyof typeof paths;

export function iconMarkup(name: IconName, className = "") {
  return `<svg class="ui-icon${className ? ` ${className}` : ""}" viewBox="0 0 24 24" aria-hidden="true" focusable="false">${paths[name] || ""}</svg>`;
}
