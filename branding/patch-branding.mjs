import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(process.argv[2] || '.');
const branding = path.resolve(process.argv[3] || path.join(root, 'branding'));
const read = p => fs.readFileSync(p, 'utf8');
const write = (p, s) => fs.writeFileSync(p, s, 'utf8');
const must = p => { if (!fs.existsSync(p)) throw new Error(`Expected upstream file not found: ${p}`); return p; };

const configPath = must(path.join(root, 'src-tauri', 'tauri.conf.json'));
const config = JSON.parse(read(configPath));
config.productName = 'Peachy Island';
config.identifier = 'app.peachy.island';
for (const win of (config.app?.windows || [])) win.title = 'Peachy Island';
if (config.bundle) {
  config.bundle.copyright = 'Peachy Island community build';
  config.bundle.shortDescription = 'A liquid-glass desktop companion for AI coding agents';
  config.bundle.longDescription = 'Peachy Island is a community-branded build of the open-source Coucou Windows app, with its agent sessions, permission approvals, chat, file drop, settings and integrations.';
}
write(configPath, JSON.stringify(config, null, 2) + '\n');

const htmlPath = must(path.join(root, 'index.html'));
write(htmlPath, read(htmlPath).replace(/<title>[^<]*<\/title>/i, '<title>Peachy Island</title>'));

const stylePath = must(path.join(root, 'src', 'style.css'));
const css = read(path.join(branding, 'peachy-overrides.css'));
const currentCss = read(stylePath);
if (!currentCss.includes('Peachy Island skin')) write(stylePath, currentCss + '\n\n' + css + '\n');

const mainPath = must(path.join(root, 'src', 'main.ts'));
let main = read(mainPath);
if (!main.includes('./peachy-branding')) main = main.replace('import "./style.css";', 'import "./style.css";\nimport "./peachy-branding";');
if (!main.includes('./peachy-branding')) main = main.replace("import './style.css';", "import './style.css';\nimport './peachy-branding';");
write(mainPath, main);
fs.copyFileSync(path.join(branding, 'peachy-branding.ts'), path.join(root, 'src', 'peachy-branding.ts'));

const iconSource = path.join(branding, 'icons');
const iconDest = path.join(root, 'src-tauri', 'icons');
fs.mkdirSync(iconDest, { recursive: true });
for (const name of ['32x32.png', '128x128.png', 'icon.ico']) {
  fs.copyFileSync(must(path.join(iconSource, name)), path.join(iconDest, name));
}
// Keep the second 128px icon expected by Tauri in sync with the new brand icon.
fs.copyFileSync(path.join(iconSource, 'icon-256.png'), path.join(iconDest, '128x128@2x.png'));

const packPath = path.join(root, 'scripts', 'pack.mjs');
if (fs.existsSync(packPath)) {
  let pack = read(packPath).replaceAll('Coucou-Windows', 'Peachy-Island-Windows');
  write(packPath, pack);
}
console.log('Peachy Island branding applied. Upstream Windows functionality remains in place.');
