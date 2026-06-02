import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/api/dialog';
import { getVersion } from '@tauri-apps/api/app';
import { open as openExternal } from '@tauri-apps/api/shell';

const elements = {
  statusBadge: document.querySelector('#statusBadge'),
  flowStateCard: document.querySelector('#flowStateCard'),
  flowStateTitle: document.querySelector('#flowStateTitle'),
  flowStateDescription: document.querySelector('#flowStateDescription'),
  parentFolder: document.querySelector('#parentFolder'),
  currentTc: document.querySelector('#currentTc'),
  capturedCount: document.querySelector('#capturedCount'),
  chooseFolderBtn: document.querySelector('#chooseFolderBtn'),
  startBtn: document.querySelector('#startBtn'),
  captureBtn: document.querySelector('#captureBtn'),
  nextTcBtn: document.querySelector('#nextTcBtn'),
  markPendingBtn: document.querySelector('#markPendingBtn'),
  finishBtn: document.querySelector('#finishBtn'),
  refreshTcListBtn: document.querySelector('#refreshTcListBtn'),
  openParentBtn: document.querySelector('#openParentBtn'),
  openTcBtn: document.querySelector('#openTcBtn'),
  tcList: document.querySelector('#tcList'),
  captureShortcut: document.querySelector('#captureShortcut'),
  nextTcShortcut: document.querySelector('#nextTcShortcut'),
  pendingShortcut: document.querySelector('#pendingShortcut'),
  finishShortcut: document.querySelector('#finishShortcut'),
  shortcutWarning: document.querySelector('#shortcutWarning'),
  lastCapturePanel: document.querySelector('#lastCapturePanel'),
  lastCaptureText: document.querySelector('#lastCaptureText'),
  appVersion: document.querySelector('#appVersion'),
  updateStatusText: document.querySelector('#updateStatusText'),
  checkUpdateBtn: document.querySelector('#checkUpdateBtn'),
  downloadUpdateBtn: document.querySelector('#downloadUpdateBtn'),
  openReleaseBtn: document.querySelector('#openReleaseBtn'),
  toast: document.querySelector('#toast')
};

const isMac = navigator.platform.toLowerCase().includes('mac');
const GITHUB_REPO = 'agungandhikaf/ss-record-rust';
const GITHUB_RELEASES_URL = `https://github.com/${GITHUB_REPO}/releases/latest`;
const GITHUB_RELEASES_API = `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`;


let state = {
  parentFolder: '',
  currentTcName: '',
  currentTcFolder: '',
  currentStep: 0,
  status: 'idle',
  tcList: [],
  lastCapture: null,
  shortcuts: {
    capture: isMac ? '⌘ + Shift + S' : 'Ctrl + Shift + S',
    nextTc: isMac ? '⌘ + Shift + N' : 'Ctrl + Shift + N',
    pending: isMac ? '⌘ + Shift + P' : 'Ctrl + Shift + P',
    finish: isMac ? '⌘ + Shift + F' : 'Ctrl + Shift + F'
  },
  shortcutStatus: [],
  appVersion: '-',
  updateInfo: {
    checking: false,
    hasUpdate: false,
    latestVersion: '',
    downloadUrl: '',
    releaseUrl: GITHUB_RELEASES_URL,
    message: 'Klik Check Update untuk mengecek versi terbaru dari GitHub Releases.'
  }
};

let busy = false;
let folderPickerBusy = false;
let toastTimer;

function showToast(message, type = 'info') {
  window.clearTimeout(toastTimer);
  elements.toast.textContent = message;
  elements.toast.classList.toggle('error', type === 'error');
  elements.toast.classList.remove('hidden');
  toastTimer = window.setTimeout(() => elements.toast.classList.add('hidden'), 3600);
}

// Feedback ringan saat window sedang diminimize: browser notification dipakai sebagai fallback OS notification.
async function notifyUser(title, body) {
  try {
    if (!('Notification' in window)) return;
    if (Notification.permission === 'default') await Notification.requestPermission();
    if (Notification.permission === 'granted') new Notification(title, { body });
  } catch (_) {
    // Notification bukan fitur utama. Kalau gagal, toast aplikasi tetap cukup.
  }
}

function allButtons() {
  return [
    elements.chooseFolderBtn,
    elements.startBtn,
    elements.captureBtn,
    elements.nextTcBtn,
    elements.markPendingBtn,
    elements.finishBtn,
    elements.refreshTcListBtn,
    elements.openParentBtn,
    elements.openTcBtn,
    elements.checkUpdateBtn,
    elements.downloadUpdateBtn,
    elements.openReleaseBtn,
    ...elements.tcList.querySelectorAll('button')
  ].filter(Boolean);
}

function setBusy(isBusy) {
  busy = isBusy;
  allButtons().forEach((button) => {
    button.disabled = isBusy;
  });
}


function normalizeVersion(version) {
  return String(version || '')
    .trim()
    .replace(/^v/i, '')
    .split('-')[0];
}

function compareVersions(a, b) {
  const left = normalizeVersion(a).split('.').map((part) => Number.parseInt(part, 10) || 0);
  const right = normalizeVersion(b).split('.').map((part) => Number.parseInt(part, 10) || 0);
  const length = Math.max(left.length, right.length);

  for (let index = 0; index < length; index += 1) {
    const diff = (left[index] || 0) - (right[index] || 0);
    if (diff !== 0) return diff;
  }

  return 0;
}

function getPlatformAsset(release) {
  const assets = Array.isArray(release?.assets) ? release.assets : [];
  const platformMatchers = isMac
    ? [/mac.*\.dmg$/i, /\.dmg$/i]
    : [/windows.*setup.*\.exe$/i, /win.*setup.*\.exe$/i, /x64.*setup.*\.exe$/i, /\.exe$/i];

  for (const matcher of platformMatchers) {
    const asset = assets.find((item) => matcher.test(item.name || ''));
    if (asset?.browser_download_url) return asset.browser_download_url;
  }

  return release?.html_url || GITHUB_RELEASES_URL;
}

function statusLabel(status) {
  const map = {
    in_progress: 'In Progress',
    pending: 'Pending',
    done: 'Done',
    empty: 'Empty'
  };
  return map[status] || status || '-';
}

function statusClass(status) {
  return ['in_progress', 'pending', 'done', 'empty'].includes(status) ? status : 'empty';
}

function getFlowUiState() {
  const hasParent = Boolean(state.parentFolder);
  const hasCurrentTc = Boolean(state.currentTcName);
  const isRecording = state.status === 'recording';
  const isFinished = state.status === 'finished';

  if (!hasParent) {
    return {
      key: 'no-folder',
      badge: 'Idle',
      title: 'Belum ada parent folder',
      description: 'Pilih parent folder dulu sebelum membuat TC001.',
      startLabel: 'Pilih Folder Dulu',
      startDisabled: true
    };
  }

  if (isRecording) {
    return {
      key: 'recording',
      badge: state.currentTcName ? `Recording ${state.currentTcName}` : 'Recording',
      title: `${state.currentTcName || 'TC'} sedang berjalan`,
      description: 'Flow sudah aktif. Sekarang gunakan Capture, Next TC, Mark Pending, atau Finish. Start dikunci agar tidak ambigu.',
      startLabel: 'Recording Aktif',
      startDisabled: true
    };
  }

  if (isFinished) {
    return {
      key: 'finished',
      badge: 'Finished',
      title: 'Flow terakhir sudah selesai',
      description: 'Klik Start TC Baru kalau mau lanjut membuat TC berikutnya pada parent folder yang sama.',
      startLabel: 'Start TC Baru',
      startDisabled: false
    };
  }

  if (hasCurrentTc) {
    return {
      key: 'resume',
      badge: 'Paused',
      title: `Session ditemukan: ${state.currentTcName}`,
      description: 'Klik Resume Flow untuk melanjutkan TC terakhir yang tersimpan.',
      startLabel: 'Resume Flow',
      startDisabled: false
    };
  }

  return {
    key: 'ready',
    badge: 'Ready',
    title: 'Siap membuat TC001',
    description: 'Klik Start Flow untuk mulai recording dan membuat folder TC001 otomatis.',
    startLabel: 'Start Flow',
    startDisabled: false
  };
}


function renderTcList() {
  const tcList = Array.isArray(state.tcList) ? state.tcList : [];

  if (!tcList.length) {
    elements.tcList.className = 'tc-list empty';
    elements.tcList.textContent = state.parentFolder
      ? 'Belum ada TC. Klik Start Flow dulu.'
      : 'Belum ada TC. Pilih parent folder lalu klik Start.';
    return;
  }

  elements.tcList.className = 'tc-list';
  elements.tcList.innerHTML = tcList
    .map((item) => {
      const isCurrent = item.tcName === state.currentTcName && state.status === 'recording';
      const canResume = state.parentFolder && item.tcName && !isCurrent;
      return `
        <div class="tc-row ${isCurrent ? 'current' : ''}">
          <div class="tc-main">
            <strong>${item.tcName}</strong>
            <span class="tc-status ${statusClass(item.status)}">${statusLabel(item.status)}</span>
          </div>
          <div class="tc-meta">
            <span>${item.stepCount || 0} screenshot</span>
            <span>${isCurrent ? 'Aktif sekarang' : item.updatedAt || ''}</span>
          </div>
          <button class="resume-btn" data-tc-name="${item.tcName}" ${canResume ? '' : 'disabled'}>${isCurrent ? 'Active' : 'Resume'}</button>
        </div>`;
    })
    .join('');
}

function render() {
  elements.parentFolder.textContent = state.parentFolder || 'Belum dipilih';
  elements.currentTc.textContent = state.currentTcName || '-';
  elements.capturedCount.textContent = String(state.currentStep || 0);

  const flowUi = getFlowUiState();
  elements.statusBadge.textContent = flowUi.badge;
  elements.statusBadge.className = `status-badge ${flowUi.key}`;
  elements.flowStateCard.className = `flow-state-card ${flowUi.key}`;
  elements.flowStateTitle.textContent = flowUi.title;
  elements.flowStateDescription.textContent = flowUi.description;
  elements.startBtn.textContent = flowUi.startLabel;
  elements.startBtn.dataset.flowState = flowUi.key;

  elements.captureShortcut.textContent = state.shortcuts.capture;
  elements.nextTcShortcut.textContent = state.shortcuts.nextTc;
  elements.pendingShortcut.textContent = state.shortcuts.pending;
  elements.finishShortcut.textContent = state.shortcuts.finish;

  const failedShortcuts = (state.shortcutStatus || []).filter((item) => !item.registered);
  if (failedShortcuts.length) {
    elements.shortcutWarning.textContent = `Ada shortcut yang gagal aktif: ${failedShortcuts.map((item) => item.accelerator).join(', ')}. Kemungkinan sedang dipakai aplikasi lain. Tombol manual tetap bisa digunakan.`;
    elements.shortcutWarning.classList.remove('hidden');
  } else {
    elements.shortcutWarning.classList.add('hidden');
  }

  if (state.lastCapture?.filePath) {
    elements.lastCaptureText.textContent = `${state.lastCapture.tcName} - Step ${state.lastCapture.step}: ${state.lastCapture.fileName}`;
    elements.lastCapturePanel.classList.remove('hidden');
  } else {
    elements.lastCapturePanel.classList.add('hidden');
  }

  elements.appVersion.textContent = state.appVersion || '-';

  if (state.updateInfo?.checking) {
    elements.updateStatusText.textContent = 'Sedang mengecek GitHub Releases...';
  } else {
    elements.updateStatusText.textContent = state.updateInfo?.message || 'Klik Check Update untuk mengecek versi terbaru.';
  }

  const hasDownloadUrl = Boolean(state.updateInfo?.downloadUrl);
  elements.downloadUpdateBtn.classList.toggle('hidden', !hasDownloadUrl || !state.updateInfo?.hasUpdate);
  elements.openReleaseBtn.classList.toggle('hidden', !state.updateInfo?.releaseUrl);

  renderTcList();

  if (!busy) {
    const hasParent = Boolean(state.parentFolder);
    const isRecording = state.status === 'recording';
    elements.startBtn.disabled = flowUi.startDisabled;
    elements.captureBtn.disabled = !isRecording;
    elements.nextTcBtn.disabled = !isRecording;
    elements.markPendingBtn.disabled = !isRecording;
    elements.finishBtn.disabled = !hasParent;
    elements.refreshTcListBtn.disabled = !hasParent;
    elements.openParentBtn.disabled = !hasParent;
    elements.openTcBtn.disabled = !state.currentTcFolder;
    elements.checkUpdateBtn.disabled = Boolean(state.updateInfo?.checking);
    elements.downloadUpdateBtn.disabled = !state.updateInfo?.downloadUrl;
    elements.openReleaseBtn.disabled = !state.updateInfo?.releaseUrl;
  }
}

async function runAction(action, successMessage) {
  if (busy) return null;
  setBusy(true);
  try {
    const result = await action();
    if (successMessage) showToast(successMessage);
    return result;
  } catch (error) {
    const message = typeof error === 'string' ? error : error?.message || 'Terjadi error.';
    showToast(message, 'error');
    await notifyUser('Flow Screenshot Recorder', message);
    return null;
  } finally {
    setBusy(false);
    render();
  }
}

async function chooseFolder() {
  // Browser folder picker dibuat khusus tanpa runAction/setBusy global.
  // Di beberapa mesin, dialog folder native Tauri bisa terasa freeze kalau seluruh UI dikunci.
  // Dengan cara ini hanya tombol Browse yang dikunci, sementara app tetap responsif.
  if (folderPickerBusy) return;

  folderPickerBusy = true;
  elements.chooseFolderBtn.disabled = true;
  elements.chooseFolderBtn.textContent = 'Membuka Folder Picker...';

  try {
    const selected = await openDialog({
      title: 'Pilih Parent Folder',
      directory: true,
      multiple: false
    });

    if (!selected) return;

    const folder = Array.isArray(selected) ? selected[0] : selected;
    if (!folder) return;

    state.parentFolder = folder;
    const restored = await invoke('read_session', { parentFolder: folder });
    state = { ...state, ...restored };
    showToast('Parent folder dipilih.');
  } catch (error) {
    const message = typeof error === 'string' ? error : error?.message || 'Gagal membuka folder picker.';
    showToast(message, 'error');
    await notifyUser('Flow Screenshot Recorder', message);
  } finally {
    folderPickerBusy = false;
    elements.chooseFolderBtn.disabled = false;
    elements.chooseFolderBtn.textContent = 'Browse';
    render();
  }
}

async function refreshSession() {
  if (!state.parentFolder) return;
  const restored = await invoke('read_session', { parentFolder: state.parentFolder });
  state = { ...state, ...restored };
  showToast('TC List diperbarui.');
  render();
}

async function startFlow() {
  if (!state.parentFolder) {
    showToast('Pilih parent folder dulu.', 'error');
    return;
  }

  const nextState = await invoke('start_or_resume_flow', { parentFolder: state.parentFolder });
  state = { ...state, ...nextState };
  showToast(`${state.currentTcName} mulai recording.`);
  render();
}

async function captureScreenshot(hideWindow = false) {
  if (!state.parentFolder || state.status !== 'recording') return;

  const capture = await invoke('capture_screenshot', {
    parentFolder: state.parentFolder,
    currentTcName: state.currentTcName,
    currentStep: state.currentStep,
    hideWindow
  });

  state = {
    ...state,
    currentStep: capture.step,
    currentTcName: capture.tcName,
    currentTcFolder: capture.tcFolder,
    status: 'recording',
    lastCapture: capture
  };

  // Refresh session setelah capture supaya step_count di TC List ikut update.
  const restored = await invoke('read_session', { parentFolder: state.parentFolder });
  state = { ...state, ...restored, lastCapture: capture };

  render();
  showToast(`Screenshot tersimpan: ${capture.fileName}`);
  await notifyUser('Screenshot tersimpan', `${capture.tcName} - Step ${capture.step}`);
}

async function nextTc() {
  if (!state.parentFolder || state.status !== 'recording') return;

  const nextState = await invoke('next_tc', {
    parentFolder: state.parentFolder,
    currentTcName: state.currentTcName
  });

  state = { ...state, ...nextState, lastCapture: null };
  showToast(`Pindah ke ${state.currentTcName}. TC sebelumnya Done.`);
  render();
}

async function markPending() {
  if (!state.parentFolder || state.status !== 'recording') return;

  const previousTc = state.currentTcName;
  const nextState = await invoke('mark_pending', {
    parentFolder: state.parentFolder,
    currentTcName: state.currentTcName
  });

  state = { ...state, ...nextState, lastCapture: null };
  showToast(`${previousTc} ditandai Pending. Lanjut ke ${state.currentTcName}.`);
  render();
}

async function resumeTc(tcName) {
  if (!state.parentFolder || !tcName) return;

  const nextState = await invoke('resume_tc', {
    parentFolder: state.parentFolder,
    targetTcName: tcName
  });

  state = { ...state, ...nextState, lastCapture: null };
  showToast(`Resume ${state.currentTcName}. Screenshot berikutnya lanjut dari step ${state.currentStep + 1}.`);
  render();
}

async function finishFlow() {
  if (!state.parentFolder) return;

  const nextState = await invoke('finish_flow', { parentFolder: state.parentFolder });
  state = { ...state, ...nextState };
  showToast('Flow selesai.');
  await notifyUser('Flow selesai', 'Semua screenshot tetap tersimpan di parent folder.');
  render();
}


async function checkForUpdate() {
  if (state.updateInfo?.checking) return;

  state = {
    ...state,
    updateInfo: {
      ...(state.updateInfo || {}),
      checking: true,
      message: 'Sedang mengecek GitHub Releases...'
    }
  };
  render();

  try {
    const response = await fetch(GITHUB_RELEASES_API, {
      cache: 'no-store',
      headers: { Accept: 'application/vnd.github+json' }
    });

    if (response.status === 404) {
      throw new Error('Belum ada GitHub Release. Buat release/tag dulu, lalu coba Check Update lagi.');
    }

    if (!response.ok) {
      throw new Error(`Gagal cek update dari GitHub. HTTP ${response.status}`);
    }

    const release = await response.json();
    const latestVersion = normalizeVersion(release.tag_name || release.name || '');
    const currentVersion = normalizeVersion(state.appVersion);
    const hasUpdate = latestVersion && compareVersions(latestVersion, currentVersion) > 0;
    const downloadUrl = hasUpdate ? getPlatformAsset(release) : '';
    const releaseUrl = release.html_url || GITHUB_RELEASES_URL;

    state = {
      ...state,
      updateInfo: {
        checking: false,
        hasUpdate,
        latestVersion,
        downloadUrl,
        releaseUrl,
        message: hasUpdate
          ? `Versi terbaru tersedia: ${latestVersion}. Current version: ${currentVersion}. Klik Download Update untuk mengambil installer terbaru.`
          : `Versi aplikasi sudah terbaru. Current version: ${currentVersion}${latestVersion ? `, latest release: ${latestVersion}` : ''}.`
      }
    };

    showToast(hasUpdate ? `Update tersedia: ${latestVersion}` : 'Aplikasi sudah versi terbaru.');
  } catch (error) {
    const message = typeof error === 'string' ? error : error?.message || 'Gagal cek update.';
    state = {
      ...state,
      updateInfo: {
        ...(state.updateInfo || {}),
        checking: false,
        hasUpdate: false,
        downloadUrl: '',
        releaseUrl: GITHUB_RELEASES_URL,
        message
      }
    };
    showToast(message, 'error');
  } finally {
    render();
  }
}

async function downloadUpdate() {
  const url = state.updateInfo?.downloadUrl || state.updateInfo?.releaseUrl || GITHUB_RELEASES_URL;
  await openExternal(url);
  showToast('Halaman/download update dibuka. Tutup aplikasi sebelum install versi baru.');
}

async function openLatestRelease() {
  await openExternal(state.updateInfo?.releaseUrl || GITHUB_RELEASES_URL);
}

function registerButtonHandlers() {
  elements.chooseFolderBtn.addEventListener('click', chooseFolder);
  elements.startBtn.addEventListener('click', () => runAction(startFlow));

  // Capture dari tombol akan hide window sebentar supaya UI aplikasi tidak ikut masuk screenshot.
  elements.captureBtn.addEventListener('click', () => runAction(() => captureScreenshot(true)));

  elements.nextTcBtn.addEventListener('click', () => runAction(nextTc));
  elements.markPendingBtn.addEventListener('click', () => runAction(markPending));
  elements.finishBtn.addEventListener('click', () => runAction(finishFlow));
  elements.refreshTcListBtn.addEventListener('click', () => runAction(refreshSession));
  elements.openParentBtn.addEventListener('click', () => runAction(() => invoke('open_path', { path: state.parentFolder })));
  elements.openTcBtn.addEventListener('click', () => runAction(() => invoke('open_path', { path: state.currentTcFolder })));
  elements.checkUpdateBtn.addEventListener('click', checkForUpdate);
  elements.downloadUpdateBtn.addEventListener('click', () => runAction(downloadUpdate));
  elements.openReleaseBtn.addEventListener('click', () => runAction(openLatestRelease));

  elements.tcList.addEventListener('click', (event) => {
    const button = event.target.closest('button[data-tc-name]');
    if (!button || button.disabled) return;
    runAction(() => resumeTc(button.dataset.tcName));
  });
}

async function registerShortcutListeners() {
  await listen('shortcut-capture', () => runAction(() => captureScreenshot(false)));
  await listen('shortcut-next-tc', () => runAction(nextTc));
  await listen('shortcut-mark-pending', () => runAction(markPending));
  await listen('shortcut-finish', () => runAction(finishFlow));

  try {
    const shortcutStatus = await invoke('get_shortcut_status');
    state.shortcutStatus = shortcutStatus;
    render();
  } catch (_) {
    // Shortcut manual lewat tombol tetap bisa dipakai walaupun status gagal dibaca.
  }
}

async function initApp() {
  registerButtonHandlers();
  registerShortcutListeners();

  try {
    state.appVersion = await getVersion();
  } catch (_) {
    state.appVersion = '-';
  }

  render();
}

initApp();
