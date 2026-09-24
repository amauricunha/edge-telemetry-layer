import json
import re

with open('docs/MBSE/EdgeTelemetryLayer_req_var.json', encoding='utf-8') as f:
    data = json.load(f)

with open('docs/MBSE/fret.md', encoding='utf-8') as f:
    md_text = f.read()

# Parse fret.md
req_blocks = re.findall(r'###\s+(REQ_\w+)[^\n]*\n(.*?)(?=\n###|\Z)', md_text, re.DOTALL)
md_map = {}
for req_id, block in req_blocks:
    vm = re.search(r'-\s*\*\*Variable Mapping:\*\*(.*?)(?=\n-\s*\*\*|\n---|\Z)', block, re.DOTALL)
    if vm:
        for line in vm.group(1).splitlines():
            m = re.search(r'-\s*`([^`]+)`\s*:\s*\*\*([^*]+)\*\*\s*\(([^)]+)\)(?:\s*[\-—–:]\s*(.*))?', line)
            if m:
                vname = m.group(1).strip()
                role = m.group(2).strip()
                vtype = m.group(3).strip()
                desc = m.group(4).strip() if m.group(4) else ''
                if vname not in md_map:
                    md_map[vname] = []
                md_map[vname].append({'req': req_id, 'role': role, 'type': vtype, 'desc': desc})

print(f"Total variables in JSON: {len(data['variables'])}")

# Analyze each JSON variable
for v in data['variables']:
    proj = v['project']
    comp = v['component_name']
    name = v['variable_name']
    
    # lookup
    lookup_name = name
    if lookup_name == 'data_packets_dispatche':
        lookup_name = 'data_packets_dispatched'
        
    occurrences = md_map.get(lookup_name, [])
    roles = set(o['role'] for o in occurrences)
    types = set(o['type'] for o in occurrences)
    descs = [o['desc'] for o in occurrences if o['desc']]
    
    # Check if anything is ambiguous
    if len(roles) > 1:
        print(f"MULTI-ROLE: {comp} -> {name}: roles={roles}")
    if len(types) > 1:
        print(f"MULTI-TYPE: {comp} -> {name}: types={types}")
