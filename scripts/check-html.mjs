// Prüft die ausgelieferten HTML-Seiten auf versehentlich als Text eingefügte Escape-Folgen.
// Hintergrund: Ein literales "\n" zwischen Tags wird als sichtbarer Text gerendert und
// zerstört das Grid-Layout der Seite. Skripte, Styles und Kommentare werden ignoriert.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const FILES = ['index.html', 'src/index.html'];

export function findLiteralEscapes(html) {
  const masked = html.replace(
    /<script\b[\s\S]*?<\/script>|<style\b[\s\S]*?<\/style>|<!--[\s\S]*?-->/gi,
    (m) => m.replace(/[^\n]/g, ' ')
  );
  const hits = [];
  const re = /\\[nrt]/g;
  let m;
  while ((m = re.exec(masked)) !== null) {
    const before = masked.slice(0, m.index);
    const line = before.split('\n').length;
    const col = m.index - before.lastIndexOf('\n');
    hits.push({ line, col, text: m[0] });
  }
  return hits;
}

export function check(files = FILES, base = root) {
  const problems = [];
  for (const file of files) {
    const full = path.join(base, file);
    if (!fs.existsSync(full)) {
      problems.push(`${file}: Datei fehlt`);
      continue;
    }
    const html = fs.readFileSync(full, 'utf8');
    for (const h of findLiteralEscapes(html)) {
      problems.push(`${file}:${h.line}:${h.col}: Literales "${h.text}" im HTML-Text. Echte Zeilenumbrüche verwenden.`);
    }
    if (!/^<!doctype html>/i.test(html.trimStart())) {
      problems.push(`${file}: Doctype fehlt am Dateianfang`);
    }
  }
  return problems;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const problems = check();
  if (problems.length) {
    console.error(problems.join('\n'));
    process.exit(1);
  }
  console.log(`HTML-Prüfung bestanden (${FILES.length} Dateien).`);
}
