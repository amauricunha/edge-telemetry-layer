const PouchDB = require('/mnt/c/workspace/fret/fret-electron/node_modules/pouchdb');
PouchDB.plugin(require('/mnt/c/workspace/fret/fret-electron/node_modules/pouchdb-find'));

const ejsCache = require('/mnt/c/workspace/fret/fret-electron/support/RealizabilityTemplates/ejsCache_realize');
const varSupports = require('/mnt/c/workspace/fret/fret-electron/model/modelDbSupport/variableMappingSupports');
const rlzUtils = require('/mnt/c/workspace/fret/fret-electron/model/realizabilitySupport/realizabilityUtils');

const fretDB = new PouchDB('/home/amauri/Documents/fret-db');
const modelDB = new PouchDB('/home/amauri/Documents/model-db');

async function test() {
  const modelRes = await modelDB.find({
    selector: {
      component_name: 'esp32s3_collector',
      completed: true,
      modeldoc: false
    }
  });
  console.log('Model docs count:', modelRes.docs.length);

  const fretRes = await fretDB.allDocs({ include_docs: true });
  console.log('Fret docs count:', fretRes.rows.length);

  const contract = varSupports.getContractInfo(modelRes);
  console.log('Contract internal variables:', contract.internalVariables);
  console.log('Contract assignments:', contract.assignments);

  const fretResult = fretRes.rows
    .map(r => r.doc)
    .filter(d => d && d.project === 'EdgeTelemetryLayer_ESPS3_COLETOR' && d.semantics && d.semantics.component_name === 'esp32s3_collector');

  console.log('Component requirements count:', fretResult.length);

  contract.properties = varSupports.getPropertyInfo(fretResult, contract.outputVariables, 'esp32s3_collector');
  contract.delays = varSupports.getDelayInfo(fretResult, 'esp32s3_collector');
  console.log('Properties count:', contract.properties.length);
  console.log('Delays:', contract.delays);

  const renamed = varSupports.variableIdentifierReplacement(contract);
  
  // Render using Kind 2
  const rendered = ejsCache.renderRealizeCode('kind2').component.complete(contract);
  console.log('Total rendered lines:', rendered.split('\n').length);

  // Check line 41433
  const lines = rendered.split('\n');
  const target = 41433;
  console.log(`--- Lines ${target - 10} to ${target + 10} ---`);
  for (let i = Math.max(0, target - 10); i < Math.min(lines.length, target + 10); i++) {
    console.log(`${i + 1}: ${lines[i]}`);
  }

  // Also write to /tmp/debug_spec.lus so we can test kind2 on it
  const fs = require('fs');
  fs.writeFileSync('/tmp/debug_spec.lus', rendered);
  console.log('Written to /tmp/debug_spec.lus');
}

test().catch(err => console.error(err));
