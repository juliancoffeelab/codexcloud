"""Prepare one output box with approved artwork as the first shape anchor."""
import argparse
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location('pinkbox_template', ROOT / '.agents/skills/pinkbox-generation/scripts/template.py')
template = importlib.util.module_from_spec(spec)
spec.loader.exec_module(template)


def prepare(base, output, labels, style_anchors, size=64, output_palette=None, flying=None, kind=None):
    source = template.matrix_path(base)
    kind = kind or ('unit' if source.stem == 'sprite' else 'prop' if 'props' in source.parts else 'terrain')
    if kind == 'unit' and flying is None:
        flying = json.loads((source.parent / 'palette.json').read_text()).get('flying', False)
    return template.prepare(output, [base, *style_anchors], labels, size, output_palette, flying=flying, kind=kind)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('base', help='Approved unit ID or artwork matrix path')
    parser.add_argument('output', type=Path)
    parser.add_argument('--label', '--labels', dest='labels', nargs=1, required=True)
    parser.add_argument('--style-anchors', nargs=2, required=True)
    parser.add_argument('--size', type=int, choices=[64], default=64)
    parser.add_argument('--kind', choices=['unit', 'terrain', 'prop'], help='Defaults to the approved matrix kind; specify prop for isolated pieces in tmp/')
    parser.add_argument('--output-palette', type=Path, help='Original material palette for the output, independently of style anchors')
    parser.add_argument('--flying', action=argparse.BooleanOptionalAction, default=None,
                        help='Override the approved unit\'s suspended-pose metadata; default preserves it')
    args = parser.parse_args()
    prepare(args.base, args.output, args.labels, args.style_anchors, args.size, args.output_palette, args.flying, args.kind)
