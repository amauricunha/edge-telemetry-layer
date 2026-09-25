import subprocess

test_js = """
const { spawnSync } = require('child_process');
let missing = [];
if (spawnSync('jkind', ['-help']).error || spawnSync('jrealizability', ['-help']).error) missing.push('jkind');
if (spawnSync('kind2', ['-h']).error) missing.push('kind2');
if (spawnSync('z3', ['-h']).error) missing.push('z3');
console.log('Missing:', missing);
let validConfigurations = [['kind2', 'z3'], ['kind2', 'z3'], ['jkind', 'z3']];
let ok = validConfigurations.some(cfg => cfg.every(dep => !missing.includes(dep)));
console.log('Realizability dependencies satisfied:', ok);
"""

# write to wsl temp file and execute
subprocess.run(['wsl', '-u', 'root', 'sh', '-c', f'echo "{test_js}" > /tmp/check_deps.js'])
res = subprocess.run(['wsl', '-d', 'Ubuntu-24.04', 'bash', '-lic', 'node /tmp/check_deps.js'], capture_output=True, text=True)
print('STDOUT:\n', res.stdout)
print('STDERR:\n', res.stderr)
