import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

const elements = {
  statusBadge: document.querySelector('#statusBadge'),
  parentFolder: document.querySelector('#parentFolder'),
  currentTc: document.querySelector('#currentTc'),
  capturedCount: document.querySelector('#capturedCount'),
  chooseFolderBtn: document.querySelector('#chooseFolderBtn'),
  startBtn: document.querySelector('#startBtn'),
  captureBtn: document.querySelector('#captureBtn'),
  nextTcBtn: document.querySelector('#nextTcBtn'),
  finishBtn: document.querySelector('#finishBtn'),
  openParentBtn: document.querySelector('#openParentBtn'),
  openTcBtn: document.querySelector('#openTcBtn'),
  captureShortcut: document.querySelector('#captureShortcut'),
  nextTcShortcut: document.querySelector('#nextTcShortcut'),
  finishShortcut: document.querySelector('#finishShortcut'),
  shortcutWarning: document.querySelector('#shortcutWarning'),
  lastCapturePanel: document.querySelector('#lastCapturePanel'),
  lastCaptureText: document.querySelector('#lastCaptureText'),
  toast: document.querySelector('#toast')
};

const isMac = navigator.platform.toLowerCase().includes('mac');

let state = {
  parentFolder: '',
  currentTcName: '',
  currentTcFolder: '',
  currentStep: 0,
  status: 'idle',
  lastCapture: null,
  shortcuts: {
    capture: isMac ? '⌘ + Shift + S' : 'Ctrl + Shift + S',
    nextTc: isMac ? '⌘ + Shift + N' : 'Ctrl + Shift + N',
    finish: isMac ? '⌘ + Shift + F' : 'Ctrl + Shift + F'
  },
  shortcutStatus: []
};

let busy = false;
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

function setBusy(isBusy) {
  busy = isBusy;
  [
    elements.chooseFolderBtn,
    elements.startBtn,
    elements.captureBtn,
    elements.nextTcBtn,
    elements.finishBtn,
    elements.openParentBtn,
    elements.openTcBtn
  ].forEach((button) => {
    button.disabled = isBusy;
  });
}

function render() {
  elements.parentFolder.textContent = state.parentFolder || 'Belum dipilih';
  elements.currentTc.textContent = state.currentTcName || '-';
  elements.capturedCount.textContent = String(state.currentStep || 0);

  elements.statusBadge.textContent = state.status === 'recording'
    ? 'Recording'
    : state.status === 'finished'
      ? 'Finished'
      : 'Idle';
  elements.statusBadge.className = `status-badge ${state.status}`;

  elements.captureShortcut.textContent = state.shortcuts.capture;
  elements.nextTcShortcut.textContent = state.shortcuts.nextTc;
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

  if (!busy) {
    const hasParent = Boolean(state.parentFolder);
    const isRecording = state.status === 'recording';
    elements.startBtn.disabled = !hasParent;
    elements.captureBtn.disabled = !isRecording;
    elements.nextTcBtn.disabled = !isRecording;
    elements.finishBtn.disabled = !hasParent;
    elements.openParentBtn.disabled = !hasParent;
    elements.openTcBtn.disabled = !state.currentTcFolder;
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
  const folder = await invoke('choose_parent_folder');
  if (!folder) return;

  state.parentFolder = folder;
  const restored = await invoke('read_session', { parentFolder: folder });
  state = { ...state, ...restored };
  showToast('Parent folder dipilih.');
  render();
}

async function startFlow() {
  if (!state.parentFolder) {
    showToast('Pilih parent folder dulu.', 'error');
    return;
  }

  const nextState = await invoke('start_or_resume_flow', { parentFolder: state.parentFolder });
  state = { ...state, ...nextState };
  showToast(`${state.currentTcName} siap dipakai.`);
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
  showToast(`Pindah ke ${state.currentTcName}.`);
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

function registerButtonHandlers() {
  elements.chooseFolderBtn.addEventListener('click', () => runAction(chooseFolder));
  elements.startBtn.addEventListener('click', () => runAction(startFlow));

  // Capture dari tombol akan hide window sebentar supaya UI aplikasi tidak ikut masuk screenshot.
  elements.captureBtn.addEventListener('click', () => runAction(() => captureScreenshot(true)));

  elements.nextTcBtn.addEventListener('click', () => runAction(nextTc));
  elements.finishBtn.addEventListener('click', () => runAction(finishFlow));
  elements.openParentBtn.addEventListener('click', () => runAction(() => invoke('open_path', { path: state.parentFolder })));
  elements.openTcBtn.addEventListener('click', () => runAction(() => invoke('open_path', { path: state.currentTcFolder })));
}

async function registerShortcutListeners() {
  await listen('shortcut-capture', () => runAction(() => captureScreenshot(false)));
  await listen('shortcut-next-tc', () => runAction(nextTc));
  await listen('shortcut-finish', () => runAction(finishFlow));

  try {
    const shortcutStatus = await invoke('get_shortcut_status');
    state.shortcutStatus = shortcutStatus;
    render();
  } catch (_) {
    // Shortcut manual lewat tombol tetap bisa dipakai walaupun status gagal dibaca.
  }
}

registerButtonHandlers();
registerShortcutListeners();
render();
