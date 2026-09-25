import json

TARGET = 'docs/MBSE/EdgeTelemetryLayer_req_var.json'

with open(TARGET, 'r', encoding='utf-8') as f:
    d = json.load(f)

count = 0
for v in d['variables']:
    comp = v['component_name']
    name = v['variable_name']
    if v['idType'] == 'Internal':
        if name == 'active_session':
            v['assignment'] = 'true'
            count += 1
        elif name == 'boot_mode':
            v['assignment'] = 'true -> false'
            count += 1
        elif name == 'connected_mode':
            v['assignment'] = 'true'
            count += 1
        elif name == 'offline_mode':
            v['assignment'] = 'false'
            count += 1
        elif name == 'buffer_occupancy':
            v['assignment'] = '0'
            count += 1

print(f"Updated {count} internal variable assignments.")

with open(TARGET, 'w', encoding='utf-8') as f:
    json.dump(d, f, indent=4, ensure_ascii=False)

print("Saved to EdgeTelemetryLayer_req_var.json!")
