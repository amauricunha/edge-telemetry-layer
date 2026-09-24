import json

with open('docs/MBSE/EdgeTelemetryLayer_req_var.json', encoding='utf-8') as f:
    data = json.load(f)

uno_vars = [v['variable_name'] for v in data['variables'] if v['component_name'] == 'uno_ecu_emulator']
esp_vars = [v['variable_name'] for v in data['variables'] if v['component_name'] == 'esp32s3_collector']

print("=== UNO VARIABLES (24) ===")
for v in sorted(uno_vars):
    print(v)

print("\n=== ESP32 VARIABLES (84) ===")
for v in sorted(esp_vars):
    print(v)
