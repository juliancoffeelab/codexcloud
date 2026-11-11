"""Declare independent native formatter rules; Dune owns execution and freshness."""
import json
import sys
for name in sorted(path for path in sys.argv[1:] if path.endswith(('.re', '.rei'))):
    target = name.replace('../', '').replace('/', '__') + '.formatted'
    print(f'''(rule
 (target {json.dumps(target)})
 (deps {json.dumps(name)})
 (action (with-stdout-to %{{target}} (run refmt --print-width 100 {json.dumps(name)}))))
(rule
 (alias fmt)
 (action (diff {json.dumps(name)} {json.dumps(target)})))''')
