const path = require('path');
const FRET_ROOT = '/mnt/c/workspace/fret/fret-electron';

const PouchDB = require(path.join(FRET_ROOT, 'app/node_modules/pouchdb'));
PouchDB.plugin(require(path.join(FRET_ROOT, 'app/node_modules/pouchdb-find')));

const fretDB = new PouchDB('/tmp/test-fret-db');
const modelDB = new PouchDB('/tmp/test-model-db');

async function test() {
  const modelRes = await modelDB.find({
    selector: {
      component_name: 'esp32s3_collector',
      completed: true,
      modeldoc: false
    }
  });
  console.log('Model docs count:', modelRes.docs.length);

  for (const doc of modelRes.docs) {
    if (doc.idType === 'Internal') {
      console.log(`Internal Var: ${doc.variable_name}, dataType=${doc.dataType}, assignment="${doc.assignment}"`);
    }
  }

  const fretRes = await fretDB.allDocs({ include_docs: true });
  console.log('Fret docs total:', fretRes.rows.length);

  const reqs = fretRes.rows
    .map(r => r.doc)
    .filter(d => d && d.project === 'EdgeTelemetryLayer_ESPS3_COLETOR' && d.semantics && d.semantics.component_name === 'esp32s3_collector');

  console.log('Component requirements count in DB:', reqs.length);
}

test().catch(err => console.error(err));
