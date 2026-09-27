// Dependency-free control reproductions, Node 24+. All assert intended behavior.
// Run: node tests/audit_controls.mjs.
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import assert from 'node:assert/strict';

const calls = [];
globalThis.__nessyAuditHooks = { call: (...args) => calls.push(args) };
const source = readFileSync(new URL('../web/ui/src/controls.ts', import.meta.url), 'utf8')
  .replace(/^import .*;\s*$/gm, '');
const js = stripTypeScriptTypes(source, { mode: 'transform' });
const { createControls, createController, Joypad } = await import(
  'data:text/javascript;base64,' + Buffer.from('const hooks = globalThis.__nessyAuditHooks;\n' + js).toString('base64')
);
const checks = [];
checks.push(['F02: restored bindings update serialized preferences', () => {
  const controls = createControls();
  const saved = JSON.parse(controls.serialize());
  saved.keyboard.inputs.up = 'z';
  controls.update(saved);
  assert.equal(controls.ref.keyboard.inputs.up, 'z');
}]);
checks.push(['F02: restored bindings remove old keys', () => {
  const controls = createControls();
  const saved = JSON.parse(controls.serialize());
  saved.keyboard.inputs.up = 'z';
  controls.update(saved);
  assert.equal(controls.isKeyMapped('w'), false);
}]);

function fixture() {
  const controls = createControls();
  const pad = { buttons: Array.from({ length: 17 }, () => ({ pressed: false })), axes: [0, 0, 0, 0] };
  Object.defineProperty(globalThis, 'navigator', { configurable: true, value: { getGamepads: () => [pad] } });
  const controller = createController({ ref: { controls } });
  controller.onGamepadConnected({ gamepad: { index: 0 } });
  return { controls, pad, controller };
}
function state() { return calls.findLast(args => args[0] === 'setJoypad1')[1]; }

checks.push(['F03: releasing a negative axis clears its button', () => {
  const { controls, pad, controller } = fixture();
  controls.setGamepad(17, 'left'); controls.setGamepad(18, 'right');
  pad.axes[0] = -1; controller.tick();
  assert.equal(state() & Joypad.LEFT, Joypad.LEFT);
  pad.axes[0] = 0; controller.tick();
  assert.equal(state() & Joypad.LEFT, 0);
}]);
checks.push(['F03: disconnect clears held gamepad buttons', () => {
  const { pad, controller } = fixture();
  pad.buttons[0].pressed = true; controller.tick();
  assert.equal(state() & Joypad.B, Joypad.B);
  controller.onGamepadDisconnected({ gamepad: { index: 0 } });
  controller.tick();
  assert.equal(state(), 0);
}]);
checks.push(['F04: Control shortcuts work and suppress key repeats', () => {
  const { controller } = fixture();
  const event = { key: 's', ctrlKey: true, metaKey: false, preventDefault() {} };
  calls.length = 0;
  controller.onKeyDown(event); controller.onKeyDown(event);
  assert.equal(calls.filter(args => args[0] === 'saveState').length, 1);
  controller.onKeyUp(event); controller.onKeyDown(event);
  assert.equal(calls.filter(args => args[0] === 'saveState').length, 2);
}]);
checks.push(['F04: gamepad button zero emits a UI press', () => {
  const { pad, controller } = fixture();
  calls.length = 0;
  pad.buttons[0].pressed = true; controller.tick();
  assert.deepEqual(calls.find(args => args[0] === 'input'), ['input', 'b', 0]);
}]);
checks.push(['F03: blur clears keyboard and gamepad input', () => {
  const { pad, controller } = fixture();
  controller.onKeyDown({key:'w', preventDefault() {}});
  pad.buttons[0].pressed = true; controller.tick();
  controller.releaseAll();
  assert.equal(state(),0);
}]);
let failures = 0;
for (const [name, check] of checks) {
  try { check(); console.log(`PASS ${name}`); }
  catch (error) { failures++; console.log(`FAIL ${name}: ${error.message}`); }
}
console.log(`${checks.length - failures}/${checks.length} control regressions passed`);
process.exitCode = failures ? 1 : 0;
