// Ground treatments sit around the original terrain features. The PNG sheets
// remain untouched; soft exclusion masks protect their trees, rocks and houses.
export const terrainDefinitions = `
  <filter id="ground-soften"><feGaussianBlur stdDeviation="1.1"/></filter>
  <pattern id="ground-grass" width="19" height="17" patternUnits="userSpaceOnUse"><path d="m3 5 1-2 1 2m9 8 1-2 1 2m-9 2 3-1" fill="none" stroke="#4c642b" stroke-opacity=".25" stroke-width=".65"/></pattern>
  <pattern id="ground-forest" width="17" height="13" patternUnits="userSpaceOnUse"><path d="m2 4 3 1m7 4 3-2m-8 5 2-1" stroke="#aaa679" stroke-opacity=".3" stroke-width=".7"/><circle cx="14" cy="3" r="1.5" fill="#273e25" fill-opacity=".35"/></pattern>
  <pattern id="ground-hill" width="23" height="19" patternUnits="userSpaceOnUse"><path d="m3 5 3-2 3 2-1 3H4z" fill="#b6b19b" fill-opacity=".5"/><path d="m16 13 2-2 3 1-1 3h-3z" fill="#706e5b" fill-opacity=".35"/><path d="m4 14 3-1m7-8 3 1" stroke="#ded0aa" stroke-opacity=".3"/></pattern>
  <pattern id="ground-village" width="16" height="12" patternUnits="userSpaceOnUse"><circle cx="3" cy="4" r=".8" fill="#ded0ab" fill-opacity=".4"/><path d="m9 8 3-1m-6 4 2-1" stroke="#7d6d45" stroke-opacity=".3" stroke-width=".8"/></pattern>
  <pattern id="ground-keep" width="12" height="8" patternUnits="userSpaceOnUse"><path d="M0 0h12v8H0zM6 0v4M0 4h12M3 4v4M9 4v4" fill="#a39d87" fill-opacity=".5" stroke="#625f54" stroke-opacity=".4" stroke-width=".6"/></pattern>
  <mask id="floor-forest"><rect width="70" height="70" fill="white"/><g fill="black" filter="url(#ground-soften)"><ellipse cx="35" cy="20" rx="15" ry="17"/><ellipse cx="18" cy="46" rx="13" ry="16"/><ellipse cx="51" cy="47" rx="14" ry="16"/></g></mask>
  <mask id="floor-hill"><rect width="70" height="70" fill="white"/><g fill="black" filter="url(#ground-soften)"><path d="M2 34 19 13 33 13 52 33 47 43H9zM33 54 44 36 53 36 65 52 62 59H33z"/></g></mask>
  <mask id="floor-village"><rect width="70" height="70" fill="white"/><g fill="black" filter="url(#ground-soften)"><path d="M6 30 21 11 32 11 41 30 39 44H9zM34 50 48 31 60 32 65 47 63 61H34z"/></g></mask>
  <mask id="floor-keep"><rect width="70" height="70" fill="white"/><path d="M16 16H54V54H16z" fill="black" filter="url(#ground-soften)"/></mask>`;

export function terrainGround(terrain, x, y, points, pixel) {
  if (terrain === 'water') return '';
  const tint = {plains:'#91a450', forest:'#344c2c', hill:'#a49b80', village:'#b99e6c', keep:'#9f9b87'}[terrain];
  const opacity = {plains:.08, forest:.43, hill:.32, village:.32, keep:.5}[terrain];
  const mask = terrain === 'plains' ? '' : `mask="url(#floor-${terrain})"`;
  const pattern = terrain === 'plains' ? 'grass' : terrain;
  const wall = terrain === 'keep' ? `<polygon class="castle-wall-shadow" points="${points}"/><polygon class="castle-wall" points="${points}"/><polygon class="castle-crenels" points="${points}"/>` : '';
  return `<svg class="terrain-ground" x="${x-35}" y="${y-35}" width="70" height="70" viewBox="0 0 70 70" aria-hidden="true" ${pixel ? 'shape-rendering="crispEdges"' : ''}><g ${mask}><rect width="70" height="70" fill="${tint}" opacity="${opacity}"/><rect width="70" height="70" fill="url(#ground-${pattern})"/></g></svg>${wall}`;
}
