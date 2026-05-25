const fs = require('fs');
const { execSync } = require('child_process');
const path = require('path');

console.log("🚀 Iniciando publicação de atualização...");

const tauriConfPath = path.join(__dirname, 'src-tauri', 'tauri.conf.json');
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
const currentVersion = tauriConf.version;

const parts = currentVersion.split('.');
parts[2] = parseInt(parts[2]) + 1;
const newVersion = parts.join('.');
console.log(`📦 Atualizando versão: ${currentVersion} -> ${newVersion}`);

tauriConf.version = newVersion;
fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2));

const packageJsonPath = path.join(__dirname, 'package.json');
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
packageJson.version = newVersion;
fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2));

console.log("🔨 Compilando aplicativo...");
execSync('npx tauri build', { stdio: 'inherit' });

console.log("🔐 Assinando...");
const exePath = path.join(__dirname, `src-tauri/target/release/bundle/nsis/FurrerTech PDF_${newVersion}_x64-setup.exe`);
const signOutput = execSync(`npx tauri signer sign -k updater-keys "${exePath}"`).toString();
const sigMatch = signOutput.match(/Signature:\s*([A-Za-z0-9+/=]+)/i) || signOutput.match(/([A-Za-z0-9+/=]+)$/m);
const signature = sigMatch ? sigMatch[1].trim() : '';

console.log("📂 Copiando para static/releases para hospedar no Vercel...");
const releasesDir = path.join(__dirname, 'static', 'releases');
if (!fs.existsSync(releasesDir)) fs.mkdirSync(releasesDir, { recursive: true });
const destExeName = `FurrerTech_PDF_${newVersion}_setup.exe`;
const destExePath = path.join(releasesDir, destExeName);
fs.copyFileSync(exePath, destExePath);

console.log("🌐 Atualizando update.json...");
const updateJsonPath = path.join(__dirname, 'static', 'update.json');
const updateJson = {
  version: newVersion,
  notes: "Atualização Automática: Melhorias gerais de performance e estabilidade.",
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: signature,
      url: `https://pdf-reader-pro-xi.vercel.app/releases/${destExeName}`
    }
  }
};
fs.writeFileSync(updateJsonPath, JSON.stringify(updateJson, null, 2));

console.log("🚀 Fazendo Deploy no Vercel...");
execSync('npx vercel --prod --yes', { stdio: 'inherit' });

console.log(`✅ SUCESSO! A versão ${newVersion} está no ar e hospedada.`);
