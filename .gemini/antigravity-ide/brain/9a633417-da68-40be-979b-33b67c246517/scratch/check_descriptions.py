import re
import json

with open('docs/MBSE/fret.md', encoding='utf-8') as f:
    text = f.read()

req_blocks = re.findall(r'###\s+(REQ_\w+)[^\n]*\n(.*?)(?=\n###|\Z)', text, re.DOTALL)

all_vars = {}
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
                if vname not in all_vars:
                    all_vars[vname] = []
                all_vars[vname].append({'req': req_id, 'role': role, 'type': vtype, 'desc': desc})

# Check how many have at least one non-empty description
vars_with_desc = {}
vars_without_desc = []
for vname, occurrences in all_vars.items():
    # find first non-empty desc
    descs = [o['desc'] for o in occurrences if o['desc']]
    roles = set(o['role'] for o in occurrences)
    types = set(o['type'] for o in occurrences)
    if descs:
        vars_with_desc[vname] = (roles, types, descs[0])
    else:
        vars_without_desc.append((vname, roles, types))

print(f'Variables WITH description in fret.md: {len(vars_with_desc)}')
print(f'Variables WITHOUT description in fret.md: {len(vars_without_desc)}')
for v in vars_without_desc:
    print('  No desc:', v)
