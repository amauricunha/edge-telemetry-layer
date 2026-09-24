# -*- coding: utf-8 -*-
import json
import copy
import sys

sys.path.append(r'c:\workspace\can-obd-telemetry\.gemini\antigravity-ide\brain\9a633417-da68-40be-979b-33b67c246517\scratch')
from var_dict_utf8 import VAR_DICT

TARGET_FILE = 'docs/MBSE/EdgeTelemetryLayer_req_var.json'

with open(TARGET_FILE, encoding='utf-8') as f:
    orig = json.load(f)

updated = copy.deepcopy(orig)

# 1. Assert requirements are not touched
assert len(orig['requirements']) == 48
assert orig['requirements'] == updated['requirements']

# 2. Update variables
updated_count = 0
for v in updated['variables']:
    comp = v['component_name']
    name = v['variable_name']
    key = (comp, name)
    assert key in VAR_DICT, f"Missing key in dictionary: {key}"
    role, dtype, desc = VAR_DICT[key]
    v['idType'] = role
    v['dataType'] = dtype
    v['description'] = desc
    v['completed'] = True
    updated_count += 1

assert updated_count == 108
assert len(updated['variables']) == 108

# 3. Double-check all other fields remain untouched
for o_var, u_var in zip(orig['variables'], updated['variables']):
    for k in o_var:
        if k in ['dataType', 'idType', 'description', 'completed']:
            continue
        assert o_var[k] == u_var[k], f"Field {k} unexpectedly changed!"

# 4. Save to target file
with open(TARGET_FILE, 'w', encoding='utf-8') as f:
    json.dump(updated, f, indent=4, ensure_ascii=False)

print("SUCCESS: EdgeTelemetryLayer_req_var.json updated safely with all 108 variables populated!")
