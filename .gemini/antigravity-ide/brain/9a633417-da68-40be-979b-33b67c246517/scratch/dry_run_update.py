# -*- coding: utf-8 -*-
import json
import copy

with open('docs/MBSE/EdgeTelemetryLayer_req_var.json', encoding='utf-8') as f:
    orig = json.load(f)

# Import dictionary
import sys
sys.path.append(r'c:\workspace\can-obd-telemetry\.gemini\antigravity-ide\brain\9a633417-da68-40be-979b-33b67c246517\scratch')
from var_dict_utf8 import VAR_DICT

updated = copy.deepcopy(orig)

# Verify requirements are untouched
assert len(orig['requirements']) == len(updated['requirements'])

# Update only variables
updated_count = 0
for v in updated['variables']:
    comp = v['component_name']
    name = v['variable_name']
    key = (comp, name)
    if key in VAR_DICT:
        role, dtype, desc = VAR_DICT[key]
        v['idType'] = role
        v['dataType'] = dtype
        v['description'] = desc
        v['completed'] = True
        updated_count += 1
    else:
        print(f"WARN: Not in dict: {key}")

print(f"Updated {updated_count} / {len(updated['variables'])} variables")

# Save to test file
with open('.gemini/antigravity-ide/brain/9a633417-da68-40be-979b-33b67c246517/scratch/test_updated.json', 'w', encoding='utf-8') as f:
    json.dump(updated, f, indent=4, ensure_ascii=False)

print("Saved test_updated.json successfully!")
