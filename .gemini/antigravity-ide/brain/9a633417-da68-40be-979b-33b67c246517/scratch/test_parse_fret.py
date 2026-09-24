import re
import json

with open('docs/MBSE/fret.md', encoding='utf-8') as f:
    text = f.read()

req_blocks = re.findall(r'###\s+(REQ_\w+)[^\n]*\n(.*?)(?=\n###|\Z)', text, re.DOTALL)
print(f'Total requirement blocks found: {len(req_blocks)}')

all_vars = {}
for req_id, block in req_blocks:
    vm = re.search(r'-\s*\*\*Variable Mapping:\*\*(.*?)(?=\n-\s*\*\*|\n---|\Z)', block, re.DOTALL)
    if vm:
        for line in vm.group(1).splitlines():
            # Match: - `var_name`: **Role** (Type) [-—–: ]? (description)?
            # Note: backtick in python file won't be escaped by powershell
            m = re.search(r'-\s*`([^`]+)`\s*:\s*\*\*([^*]+)\*\*\s*\(([^)]+)\)(?:\s*[\-—–:]\s*(.*))?', line)
            if m:
                vname = m.group(1).strip()
                role = m.group(2).strip()
                vtype = m.group(3).strip()
                desc = m.group(4).strip() if m.group(4) else ''
                if vname not in all_vars:
                    all_vars[vname] = []
                all_vars[vname].append({'req': req_id, 'role': role, 'type': vtype, 'desc': desc})

print(f'Total distinct variables extracted from fret.md: {len(all_vars)}')

with open('docs/MBSE/EdgeTelemetryLayer_req_var.json', encoding='utf-8') as f:
    jdata = json.load(f)

json_vars = jdata.get('variables', [])
json_vnames = set(v['variable_name'] for v in json_vars)
print(f'Distinct variable names in JSON: {len(json_vnames)}')

missing_in_md = json_vnames - set(all_vars.keys())
print(f'Variables in JSON but not in fret.md Variable Mappings: {missing_in_md}')

missing_in_json = set(all_vars.keys()) - json_vnames
print(f'Variables in fret.md Variable Mappings but not in JSON: {missing_in_json}')
