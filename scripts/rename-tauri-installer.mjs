import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

// [VERSIONED_INSTALLER_FIX]
// Tauri membuat nama installer default seperti "MyTBC_1.0.8_x64-setup.exe".
// Script ini membuat salinan release-friendly sesuai format yang diminta:
// - MyTBC_x64_v.1.0.8.exe
// - MyTBC_mac_v.1.0.8.dmg

const ROOT_DIR = fileURLToPath(new URL('..', import.meta.url));
const DIST_DIR = path.join(ROOT_DIR, 'dist-installers');
const TARGET_DIR = path.join(ROOT_DIR, 'src-tauri', 'target', 'release', 'bundle');

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function getAppVersion() {
  const tauriConfigPath = path.join(ROOT_DIR, 'src-tauri', 'tauri.conf.json');
  const packageJsonPath = path.join(ROOT_DIR, 'package.json');

  try {
    const tauriConfig = readJson(tauriConfigPath);
    const version = tauriConfig?.package?.version;
    if (version) return String(version).trim();
  } catch (_) {
    // Fallback ke package.json jika config Tauri tidak bisa dibaca.
  }

  const packageJson = readJson(packageJsonPath);
  return String(packageJson.version || '').trim();
}

function listFilesRecursive(directory, extension) {
  if (!fs.existsSync(directory)) return [];

  const files = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...listFilesRecursive(fullPath, extension));
    } else if (entry.isFile() && entry.name.toLowerCase().endsWith(extension)) {
      files.push(fullPath);
    }
  }

  return files;
}

function newestFile(files) {
  return files
    .map((filePath) => ({ filePath, modifiedTime: fs.statSync(filePath).mtimeMs }))
    .sort((a, b) => b.modifiedTime - a.modifiedTime)[0]?.filePath;
}

function copyVersionedInstaller(platform) {
  const version = getAppVersion();
  if (!version) {
    throw new Error('Versi aplikasi tidak ditemukan di src-tauri/tauri.conf.json atau package.json.');
  }

  const configByPlatform = {
    win: {
      sourceDir: path.join(TARGET_DIR, 'nsis'),
      extension: '.exe',
      outputName: `MyTBC_x64_v.${version}.exe`
    },
    mac: {
      sourceDir: path.join(TARGET_DIR, 'dmg'),
      extension: '.dmg',
      outputName: `MyTBC_mac_v.${version}.dmg`
    }
  };

  const config = configByPlatform[platform];
  if (!config) {
    throw new Error('Platform tidak valid. Gunakan: win atau mac.');
  }

  const sourceFiles = listFilesRecursive(config.sourceDir, config.extension);
  const sourceFile = newestFile(sourceFiles);

  if (!sourceFile) {
    throw new Error(`Installer ${config.extension} tidak ditemukan di: ${config.sourceDir}`);
  }

  fs.mkdirSync(DIST_DIR, { recursive: true });
  const outputPath = path.join(DIST_DIR, config.outputName);
  fs.copyFileSync(sourceFile, outputPath);

  console.log('[VERSIONED_INSTALLER_FIX] Installer original :', path.relative(ROOT_DIR, sourceFile));
  console.log('[VERSIONED_INSTALLER_FIX] Installer versioned:', path.relative(ROOT_DIR, outputPath));
}

try {
  copyVersionedInstaller(process.argv[2]);
} catch (error) {
  console.error('[VERSIONED_INSTALLER_FIX] Gagal membuat nama installer versi:', error.message);
  process.exit(1);
}
