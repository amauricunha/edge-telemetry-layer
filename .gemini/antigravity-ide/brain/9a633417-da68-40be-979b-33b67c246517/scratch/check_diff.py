import json

orig = json.load(open('docs/MBSE/EdgeTelemetryLayer_req_var.json', encoding='utf-8'))
up = json.load(open('.gemini/antigravity-ide/brain/9a633417-da68-40be-979b-33b67c246517/scratch/test_updated.json', encoding='utf-8'))

assert orig['requirements'] == up['requirements'], 'Requirements were touched!'
print('REQUIREMENTS CHECK: 100% IDENTICAL (UNTOUCHED)')

for o_var, u_var in zip(orig['variables'], up['variables']):
    for k in o_var:
        if k in ['dataType', 'idType', 'description', 'completed']:
            continue
        assert o_var[k] == u_var[k], f'Key {k} changed in var {o_var["variable_name"]}'

print('VARIABLES METADATA CHECK: ALL OTHER FIELDS 100% IDENTICAL')
print('Sample updated variable:')
print(json.dumps(up['variables'][0], indent=2, ensure_ascii=False))
