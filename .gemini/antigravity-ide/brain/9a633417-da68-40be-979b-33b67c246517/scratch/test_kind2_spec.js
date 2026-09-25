const path = require('path');
const FRET_ROOT = '/mnt/c/workspace/fret/fret-electron';

const ejsCache = require(path.join(FRET_ROOT, 'support/RealizabilityTemplates/ejsCache_realize'));
const varSupports = require(path.join(FRET_ROOT, 'model/modelDbSupport/variableMappingSupports'));

const PouchDB = require(path.join(FRET_ROOT, 'app/node_modules/pouchdb'));
PouchDB.plugin(require(path.join(FRET_ROOT, 'app/node_modules/pouchdb-find')));

const fretDB = new PouchDB('/tmp/test-fret-db');
const modelDB = new PouchDB('/tmp/test-model-db');

const { spawnSync } = require('child_process');
const fs = require('fs');

async function run() {
  const modelRes = await modelDB.find({
    selector: {
      component_name: 'esp32s3_collector',
      completed: true,
      modeldoc: false
    }
  });

  const fretRes = await fretDB.allDocs({ include_docs: true });
  const fretResult = fretRes.rows
    .map(r => r.doc)
    .filter(d => d && d.project === 'EdgeTelemetryLayer_ESPS3_COLETOR' && d.semantics && d.semantics.component_name === 'esp32s3_collector');

  console.log(`Loaded ${modelRes.docs.length} variables and ${fretResult.length} requirements.`);

  // Test Case 1: As-is (empty assignments)
  console.log('\n--- TEST 1: Current DB state (empty assignments) ---');
  let contract1 = varSupports.getContractInfo(modelRes);
  contract1.componentName = 'esp32s3_collectorSpec';
  contract1.properties = varSupports.getPropertyInfo(fretResult, contract1.outputVariables, 'esp32s3_collector');
  contract1.delays = varSupports.getDelayInfo(fretResult, 'esp32s3_collector');
  contract1 = varSupports.variableIdentifierReplacement(contract1);

  let rendered1 = ejsCache.renderRealizeCode('kind2').component.complete(contract1);
  fs.writeFileSync('/tmp/test1.lus', rendered1);
  let res1 = spawnSync('kind2', ['--lus_main', 'esp32s3_collectorSpec', '/tmp/test1.lus'], { encoding: 'utf8' });
  console.log('Test 1 Kind2 Exit Code:', res1.status);
  console.log('Test 1 Kind2 Output head:', (res1.stdout || res1.stderr).substring(0, 300));

  // Test Case 2: Set assignments for Internal variables
  console.log('\n--- TEST 2: With Lustre assignments for Internal variables ---');
  let modelRes2 = JSON.parse(JSON.stringify(modelRes));
  for (let doc of modelRes2.docs) {
    if (doc.idType === 'Internal') {
      if (doc.variable_name === 'active_session') doc.assignment = 'true';
      else if (doc.variable_name === 'boot_mode') doc.assignment = 'true -> false';
      else if (doc.variable_name === 'connected_mode') doc.assignment = 'true';
      else if (doc.variable_name === 'offline_mode') doc.assignment = 'false';
      else if (doc.variable_name === 'buffer_occupancy') doc.assignment = '0';
    }
  }

  let contract2 = varSupports.getContractInfo(modelRes2);
  contract2.componentName = 'esp32s3_collectorSpec';
  contract2.properties = varSupports.getPropertyInfo(fretResult, contract2.outputVariables, 'esp32s3_collector');
  contract2.delays = varSupports.getDelayInfo(fretResult, 'esp32s3_collector');
  contract2 = varSupports.variableIdentifierReplacement(contract2);

  let rendered2 = ejsCache.renderRealizeCode('kind2').component.complete(contract2);
  fs.writeFileSync('/tmp/test2.lus', rendered2);
  let res2 = spawnSync('kind2', ['--lus_main', 'esp32s3_collectorSpec', '/tmp/test2.lus'], { encoding: 'utf8' });
  console.log('Test 2 Kind2 Exit Code:', res2.status);
  console.log('Test 2 Kind2 Output head:\n', (res2.stdout || res2.stderr).substring(0, 400));
}

run().catch(err => console.error(err));
