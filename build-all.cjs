const fs = require('fs');
const { execSync } = require('child_process');
const path = require('path');

const root = __dirname;
const tauriConfPath = path.join(root, 'src-tauri', 'tauri.conf.json');

// Ensure public key is correctly set in tauri.conf.json
const pubKey = fs.readFileSync(path.join(root, 'updater-keys.pub'), 'utf8').trim();
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
tauriConf.plugins = tauriConf.plugins || {};
tauriConf.plugins.updater = {
  pubkey: pubKey,
  endpoints: ["https://pdf-reader-pro-xi.vercel.app/update.json"]
};
tauriConf.version = "0.1.1"; // Base version
fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2));

const packageJsonPath = path.join(root, 'package.json');
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
packageJson.version = "0.1.1";
fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2));

console.log("🏗️ Construindo v0.1.1 BASE (A que o usuário vai instalar agora)...");
execSync('npx tauri build', { stdio: 'inherit', cwd: root });

console.log("🚀 Executando release-update.js para criar v0.1.2 simulando uma atualização futura...");
execSync('node release-update.js', { stdio: 'inherit', cwd: root });

console.log("Tudo pronto!");
