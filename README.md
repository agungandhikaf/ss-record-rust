# Flow Screenshot Recorder — Tauri + Rust

Aplikasi desktop ringan untuk membuat evidence screenshot per test case.

## Fitur

- Pilih parent folder saja.
- Auto create folder `TC001`, `TC002`, `TC003`, dst.
- Screenshot manual via tombol atau shortcut.
- Mode screenshot **Browser Area**:
  - Windows: crop dari area address/search bar pada window aktif sehingga taskbar tidak ikut.
  - macOS/Linux: fallback crop monitor utama dengan offset tetap.
- `Next TC` menandai TC aktif sebagai `Done`, lalu lanjut TC baru.
- `Mark Pending` menandai TC aktif sebagai `Pending`, lalu lanjut TC baru.
- `Resume` dari TC List untuk melanjutkan TC lama/pending.
- Autosave nama file unik.
- `metadata.json` per TC.
- `session.json` untuk resume saat aplikasi ditutup lalu dibuka lagi.

## Shortcut

- Windows/Linux: `Ctrl + Shift + S` = Capture
- Windows/Linux: `Ctrl + Shift + N` = Next TC
- Windows/Linux: `Ctrl + Shift + P` = Mark Pending
- Windows/Linux: `Ctrl + Shift + F` = Finish

- Mac: `Command + Shift + S` = Capture
- Mac: `Command + Shift + N` = Next TC
- Mac: `Command + Shift + P` = Mark Pending
- Mac: `Command + Shift + F` = Finish

## Cara run development

```bash
npm install
npm run dev
```

## Build installer

Windows:

```bash
npm run dist:win
```

Mac:

```bash
npm run dist:mac
```

## GitHub Actions

Workflow sudah tersedia di:

```text
.github/workflows/build-installers.yml
```

Upload project ini ke GitHub, lalu buka:

```text
Actions > Build Tauri Installers > Run workflow
```

Hasil build bisa diambil dari bagian `Artifacts`:

- `tauri-windows-installer`
- `tauri-mac-installer`

## Struktur output

```text
Parent Folder/
├── TC001/
│   ├── 001_20260602_103020_123_abcd.png
│   └── metadata.json
├── TC002/
│   ├── 001_20260602_103300_456_ef01.png
│   └── metadata.json
└── session.json
```

## Catatan crop screenshot

Mode Browser Area cocok untuk flow browser yang window-nya aktif. Untuk Windows, aplikasi mencoba membaca window aktif dan crop dari area address/search bar. Kalau hasil crop terlalu atas atau terlalu bawah, ubah konstanta di `src-tauri/src/main.rs` pada fungsi:

```rust
fn browser_crop_insets() -> (u32, u32)
```

Nilai pertama adalah crop atas. Makin besar nilainya, makin banyak bagian atas yang dipotong.
