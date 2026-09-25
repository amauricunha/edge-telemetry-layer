import os

# 1. Patch variableMappingSupports.js
vms_path = r'c:\workspace\fret\fret-electron\model\modelDbSupport\variableMappingSupports.js'
with open(vms_path, 'r', encoding='utf-8') as f:
    vms_content = f.read()

old_getDelayInfo = '''function getDelayInfo(result, component) {
  var delays = [];
  result.docs.forEach(function(doc){
    if (doc.semantics && doc.semantics.component_name === component){
      if (typeof doc.semantics.CoCoSpecCode !== 'undefined'){
        if (doc.semantics.CoCoSpecCode !== constants.nonsense_semantics &&
          doc.semantics.CoCoSpecCode !== constants.undefined_semantics &&
          doc.semantics.CoCoSpecCode !== constants.unhandled_semantics){
            if (doc.semantics.duration){
                if (!delays.includes(doc.semantics.duration)){
                  delays.push(doc.semantics.duration);
                }
            }
        }
      }
    }
  })
  return delays;
}'''

new_getDelayInfo = '''function getDelayInfo(result, component, selectedReqs) {
  var delays = [];
  result.docs.forEach(function(doc){
    if (doc.semantics && doc.semantics.component_name === component){
      var reqid = doc.reqid ? doc.reqid.replace(/-/g, '') : '';
      if (!selectedReqs || selectedReqs.length === 0 || selectedReqs.includes(reqid) || selectedReqs.includes(doc.reqid)){
        if (typeof doc.semantics.CoCoSpecCode !== 'undefined'){
          if (doc.semantics.CoCoSpecCode !== constants.nonsense_semantics &&
            doc.semantics.CoCoSpecCode !== constants.undefined_semantics &&
            doc.semantics.CoCoSpecCode !== constants.unhandled_semantics){
              if (doc.semantics.duration){
                  if (!delays.includes(doc.semantics.duration)){
                    delays.push(doc.semantics.duration);
                  }
              }
          }
        }
      }
    }
  })
  return delays;
}'''

assert old_getDelayInfo in vms_content, "old_getDelayInfo not found in variableMappingSupports.js"
vms_content = vms_content.replace(old_getDelayInfo, new_getDelayInfo, 1)
with open(vms_path, 'w', encoding='utf-8', newline='\n') as f:
    f.write(vms_content)
print("variableMappingSupports.js patched successfully!")

# 2. Patch realizabilityUtils.js
ru_path = r'c:\workspace\fret\fret-electron\model\realizabilitySupport\realizabilityUtils.js'
with open(ru_path, 'r', encoding='utf-8') as f:
    ru_content = f.read()

# Monolithic part
old_mono = '''        if (monolithic) {
            //We add the 'Spec' suffix to avoid clashes with potential Lustre keywords.
            var specName = tC.component_name + 'Spec';
            contract.componentName = specName;
            contract.properties = contract.properties.filter(p => selectedReqs.includes(p.reqid.substring(2)))
            var filePath = analysisPath + specName +'.lus';'''

new_mono = '''        if (monolithic) {
            //We add the 'Spec' suffix to avoid clashes with potential Lustre keywords.
            var specName = tC.component_name + 'Spec';
            contract.componentName = specName;
            contract.properties = contract.properties.filter(p => selectedReqs.includes(p.reqid.substring(2)))
            contract.delays = getDelayInfo(fretResult, tC.component_name, selectedReqs);
            var filePath = analysisPath + specName +'.lus';'''

assert old_mono in ru_content, "old_mono not found in realizabilityUtils.js"
ru_content = ru_content.replace(old_mono, new_mono, 1)

# Compositional part
old_comp = '''            var ccProperties = contract.properties.filter(p => cc.requirements.includes(p.reqid.substring(2)));

            ccContract.properties = (cc.ccName === ccSelected) ? ccProperties.filter(p => selectedReqs.includes(p.reqid.substring(2))) : ccProperties;
            var lustreContract = ejsCache_realize.renderRealizeCode(engineName).component.complete(ccContract);'''

new_comp = '''            var ccProperties = contract.properties.filter(p => cc.requirements.includes(p.reqid.substring(2)));

            ccContract.properties = (cc.ccName === ccSelected) ? ccProperties.filter(p => selectedReqs.includes(p.reqid.substring(2))) : ccProperties;
            var targetReqs = (cc.ccName === ccSelected) ? selectedReqs : cc.requirements;
            ccContract.delays = getDelayInfo(fretResult, tC.component_name, targetReqs);
            var lustreContract = ejsCache_realize.renderRealizeCode(engineName).component.complete(ccContract);'''

assert old_comp in ru_content, "old_comp not found in realizabilityUtils.js"
ru_content = ru_content.replace(old_comp, new_comp, 1)

with open(ru_path, 'w', encoding='utf-8', newline='\n') as f:
    f.write(ru_content)
print("realizabilityUtils.js patched successfully!")
