#!/usr/bin/env python3
"""Embed package-owned frozen JSON without foreign module imports."""
import json
from pathlib import Path
import re
import sys


def generate(package: Path, output: Path) -> None:
    sources = sorted((package / "src/legacy/data").glob("*.json"))
    sources.append(package / "src/formats/readers.json")
    declarations = [
        "(* Generated from frozen package data. Each value is parsed once, on first use. *)"
    ]
    for source in sources:
        category = "formats" if source.parent.name == "formats" else "history"
        name = category + "_" + re.sub(r"[^a-z0-9_]", "_", source.stem)
        # ASCII keeps JSON's Unicode semantics independent of OCaml byte strings.
        data = json.dumps(
            json.loads(source.read_text()), ensure_ascii=True, separators=(",", ":"), allow_nan=False
        )
        delimiter = "frozen_json"
        while "|" + delimiter + "}" in data:
            delimiter += "_"
        declarations.append(
            f"let {name} : Js.Json.t Lazy.t =\n"
            f"  lazy (Js.Json.parseExn {{{delimiter}|{data}|{delimiter}}})"
        )
    output.write_text("\n\n".join(declarations) + "\n")


if __name__ == "__main__":
    generate(Path(sys.argv[1]), Path(sys.argv[2]))
