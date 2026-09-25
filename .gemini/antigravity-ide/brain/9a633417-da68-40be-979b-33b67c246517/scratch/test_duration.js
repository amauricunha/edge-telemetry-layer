const parser = require('/mnt/c/workspace/fret/fret-electron/app/parser/RequirementParser');
const lexer = require('/mnt/c/workspace/fret/fret-electron/app/parser/RequirementLexer');
const antlr4 = require('/mnt/c/workspace/fret/fret-electron/node_modules/antlr4');
const FretSemantics = require('/mnt/c/workspace/fret/fret-electron/app/parser/FretSemantics');

const textInt = "in active_session upon physics_model_updated the uno_ecu_emulator shall within 2 MILLISECOND satisfy dbc_frames_emitted";
const textFloat = "in active_session upon physics_model_updated the uno_ecu_emulator shall within 2.0 MILLISECOND satisfy dbc_frames_emitted";

console.log("--- Testing Semantics generation ---");
try {
  const semInt = new FretSemantics().compileToString(textInt);
  console.log("2 MILLISECOND CoCoSpecCode:", semInt.CoCoSpecCode);
} catch (e) {
  console.log("Error with 2:", e.message);
}

try {
  const semFloat = new FretSemantics().compileToString(textFloat);
  console.log("2.0 MILLISECOND CoCoSpecCode:", semFloat.CoCoSpecCode);
} catch (e) {
  console.log("Error with 2.0:", e.message);
}
