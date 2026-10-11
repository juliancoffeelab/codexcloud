"""Prepare one output box with the approved unit as the first anatomy anchor."""
import argparse
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location('pinkbox_template', ROOT / '.agents/skills/pinkbox-generation/scripts/template.py')
template = importlib.util.module_from_spec(spec)
spec.loader.exec_module(template)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('base', help='Approved unit ID or shared sprite.txt path')
    parser.add_argument('output', type=Path)
    parser.add_argument('--label', '--labels', dest='labels', nargs=1, required=True)
    parser.add_argument('--style-anchors', nargs=2, required=True)
    parser.add_argument('--size', type=int, choices=[64], default=64)
    parser.add_argument('--output-palette', type=Path, help='Original material palette for the output, independently of style anchors')
    args = parser.parse_args()
    base = template.matrix_path(args.base)
    rows = template.render.read_matrix(base)
    size = args.size
    template.prepare(args.output, [args.base, *args.style_anchors], args.labels, size, args.output_palette)
