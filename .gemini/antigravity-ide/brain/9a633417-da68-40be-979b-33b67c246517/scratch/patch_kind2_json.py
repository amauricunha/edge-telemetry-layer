import os

path = r'c:\workspace\fret\fret-electron\analysis\realizabilityCheck.js'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

target = '    var jsonContent = JSON.parse(stdout);'
replacement = '''    var jsonContent;
    try {
      jsonContent = JSON.parse(stdout);
    } catch (parseErr) {
      callback(new Error('Kind 2 output parse error: ' + parseErr.message));
      return;
    }'''

if target in content:
    content = content.replace(target, replacement, 1)
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        f.write(content)
    print("Patch applied successfully!")
else:
    print("Target already patched or not found.")
