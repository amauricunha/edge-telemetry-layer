import json

with open('docs/MBSE/EdgeTelemetryLayer_req_var.json', encoding='utf-8') as f:
    d = json.load(f)

for r in d['requirements']:
    sem = r.get('semantics', {})
    comp = sem.get('component_name', '')
    timing = sem.get('timing', '')
    dur = sem.get('duration', '')
    ft = r.get('fulltext', '')
    print(f"{r['reqid']} ({comp}): duration={dur}, fulltext={ft}")
