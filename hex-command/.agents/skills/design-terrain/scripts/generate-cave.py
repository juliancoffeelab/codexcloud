"""Author native cave passage frames from explicit open-edge definitions, without map input."""
import argparse
import json
import math
from PIL import Image, ImageFilter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
DIRECTIONS = ['e', 'se', 'sw', 'w', 'nw', 'ne']
APOTHEM = 16 * math.sqrt(3)
PALETTE = {
    '.': [0, 0, 0, 0],
    'R': [44, 48, 55, 255],
    'r': [60, 65, 73, 255],
    'M': [83, 89, 97, 255],
    'm': [99, 105, 111, 255],
    'L': [129, 133, 135, 255],
    'l': [111, 117, 123, 255],
    'S': [30, 34, 40, 255],
}


def definitions(path):
    variants = json.loads(path.read_text())
    masks = set()
    for variant in variants:
        edges = variant.get('connections')
        if (set(variant) != {'id', 'connections'} or not isinstance(edges, list)
                or len(set(edges)) != len(edges) or any(edge not in DIRECTIONS for edge in edges)):
            raise ValueError('Each cave variant requires unique explicit connections')
        mask = sum(1 << DIRECTIONS.index(edge) for edge in edges)
        if variant['id'] != str(mask) or mask in masks:
            raise ValueError('Invalid or duplicate cave variant')
        masks.add(mask)
    if masks != set(range(64)):
        raise ValueError('Cave library must define all 64 open-edge configurations')
    return variants


def hashed(x,y,seed):
    n=(x*374761393+y*668265263+seed*1442695041)&0xffffffff
    n=((n^(n>>13))*1274126177)&0xffffffff
    return ((n^(n>>16))&0xffff)/65535

def noise(x,y,frequency,seed):
    u=((x/56-y/96)%1)*frequency; v=((y/48)%1)*frequency
    ix,iy=math.floor(u),math.floor(v); fx,fy=u-ix,v-iy
    fx=fx*fx*(3-2*fx); fy=fy*fy*(3-2*fy)
    def value(dx,dy): return hashed((ix+dx)%frequency,(iy+dy)%frequency,seed)
    return (value(0,0)*(1-fx)+value(1,0)*fx)*(1-fy)+(value(0,1)*(1-fx)+value(1,1)*fx)*fy

def ground(x,y):
    patch=.70*noise(x,y,5,11)+.30*noise(x,y,11,17)
    grit=noise(x,y,23,37)
    if grit>.82: return 'C'
    if grit<.13: return 'D'
    if patch<.34: return 'g'
    if patch<.48: return 'A'
    if patch>.69: return 'B'
    return 'G'

PALETTE.update({'A':[73,77,83,255], 'B':[88,91,96,255],
                'C':[104,107,112,255], 'D':[56,60,67,255]})

def frame(mask, base):
    footprint={(x-32,y-32) for y,row in enumerate(base) for x,s in enumerate(row) if s!='.'}
    shifts=[(56,0),(28,48),(-28,48),(-56,0),(-28,-48),(28,-48)]
    # Every frame is authored independently from its explicit connection mask.
    # Open-end profiles continue through the shared corner at a fixed height.
    domain=set(footprint)
    for edge,(dx,dy) in enumerate(shifts):
        if mask & (1<<edge): domain.update((x+dx,y+dy) for x,y in footprint)
    stencil=Image.new('L',(200,240))
    stencil.putdata([255 if (x-100,y-120) in domain else 0 for y in range(240) for x in range(200)])
    # Modulate the inward rock contour and its cap height in coherent lumps.
    inside=[stencil.filter(ImageFilter.MinFilter(size)) for size in (9,11,13,15,17)]
    heights={}
    for x,y in domain:
        band=min(4,int(noise(x,y,9,51)*5))
        edge=inside[band].getpixel((x+100,y+120))==0
        heights[x,y]=(5+int(noise(x,y,12,61)*3)) if edge else 0
    raster={p:(ground(p[0],p[1]),p) for p in domain}
    for x,y in sorted(domain,key=lambda p:(p[1],p[0])):
        h=heights[x,y]
        if not h: continue
        for z in range(h+1):
            # Periodic coordinates match under every 56x48 lattice translation.
            grain=int(noise(x,y,16,73)*11)
            symbol=('M' if grain<6 else 'm') if z==h else ('l' if grain>6 else 'M') if z==h-1 else 'S' if z==0 else 'r' if grain<7 else 'R'
            raster[x,y-z]=(symbol,(x,y))
    rows=[['.']*80 for _ in range(96)]
    for (x,y),(symbol,origin) in raster.items():
        # The finished frame owns its full screen-space hex, including the baked
        # continuation at open corners. Exterior silhouettes belong to the wall
        # that produced them; no floor may erase another frame's raised edge.
        if (x,y) in footprint or ((x,y) not in domain and origin in footprint):
            px,py=x+40,y+48
            if 0<=px<80 and 0<=py<96: rows[py][px]=symbol
    rows=[''.join(row) for row in rows]
    return rows

def wall_frame(base):
    """Complete solid-rock hex; no walkable floor or map-dependent geometry."""
    rows = []
    for y, source in enumerate(base):
        row = []
        for x, pixel in enumerate(source):
            if pixel == '.':
                row.append('.')
                continue
            patch = noise(x - 32, y - 32, 7, 89)
            crack = noise(x - 32, y - 32, 19, 97)
            row.append('S' if crack < .16 else 'R' if patch < .38 else 'r' if patch < .62 else 'M' if patch < .78 else 'm')
        rows.append(''.join(row))
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--catalog', type=Path, required=True)
    parser.add_argument('--walls-output', type=Path, help='Optional directory for a solid impassable rock tile')
    args = parser.parse_args()
    variants = definitions(args.catalog)
    base = (ROOT / 'pixel/art/terrain/plains/tile.txt').read_text().splitlines()
    args.output.mkdir(parents=True, exist_ok=True)
    for variant in variants:
        rows = frame(int(variant['id']), base)
        (args.output / f"{variant['id']}.txt").write_text('\n'.join(rows) + '\n')
    (args.output / 'palette.json').write_text(json.dumps(PALETTE, indent=2) + '\n')
    (args.output / 'canvas.json').write_text(json.dumps({'width':80,'height':96,'anchorX':40,'anchorY':48,'z':1},indent=2)+'\n')
    if args.walls_output is not None:
        args.walls_output.mkdir(parents=True, exist_ok=True)
        (args.walls_output / 'tile.txt').write_text('\n'.join(wall_frame(base)) + '\n')
        (args.walls_output / 'palette.json').write_text(json.dumps({s: PALETTE[s] for s in '.SRrMm'}, indent=2) + '\n')
    print(f'Authored {len(variants)} complete native cave passage frames on the castle canvas.')


if __name__ == '__main__':
    main()
