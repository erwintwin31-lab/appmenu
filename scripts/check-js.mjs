// Syntax-check all shipped frontend and local verification scripts without adding dependencies.
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
function walk(dir) {
  return fs.readdirSync(dir, {withFileTypes:true}).flatMap(entry => {
    const full = path.join(dir, entry.name);
    if (entry.isSymbolicLink()) return [];
    if (entry.isDirectory()) return walk(full);
    return /\.(?:js|mjs|cjs)$/.test(entry.name) ? [full] : [];
  });
}
const files = ['src','demo','scripts','tests'].flatMap(name => {
  const dir = path.join(root,name); return fs.existsSync(dir) ? walk(dir) : [];
}).sort();
let failed = false;
for (const file of files) {
  const result = spawnSync(process.execPath, ['--check',file], {stdio:'inherit'});
  if (result.error || result.status !== 0) { console.error(path.relative(root,file)); failed = true; }
}
if (failed) process.exit(1);
console.log(`JavaScript-Syntaxprüfung bestanden (${files.length} Dateien).`);
