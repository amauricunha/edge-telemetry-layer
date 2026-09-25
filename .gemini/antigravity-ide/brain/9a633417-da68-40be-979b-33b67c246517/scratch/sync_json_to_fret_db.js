const path = require('path');
const fs = require('fs');

const FRET_ROOT = '/mnt/c/workspace/fret/fret-electron';
const PouchDB = require(path.join(FRET_ROOT, 'app/node_modules/pouchdb'));
PouchDB.plugin(require(path.join(FRET_ROOT, 'app/node_modules/pouchdb-find')));

const modelDB = new PouchDB('/home/amauri/Documents/model-db');

const JSON_PATH = '/mnt/c/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_req_var.json';
const jsonData = JSON.parse(fs.readFileSync(JSON_PATH, 'utf8'));

async function sync() {
  console.log('Reading JSON variables...');
  let updatedCount = 0;

  for (const v of jsonData.variables) {
    if (v.idType === 'Internal') {
      const res = await modelDB.find({
        selector: {
          project: v.project,
          component_name: v.component_name,
          variable_name: v.variable_name
        }
      });

      for (const doc of res.docs) {
        doc.assignment = v.assignment;
        doc.idType = v.idType;
        doc.dataType = v.dataType;
        doc.description = v.description;
        doc.completed = true;
        await modelDB.put(doc);
        console.log(`Updated in model-db: ${v.component_name} -> ${v.variable_name} = "${v.assignment}"`);
        updatedCount++;
      }
    }
  }

  console.log(`Sync completed! ${updatedCount} records updated in model-db.`);
}

sync().catch(err => {
  console.error('Error syncing DB (FRET may be open):', err.message);
});
