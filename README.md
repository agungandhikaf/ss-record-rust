# MyScreenshots — Tauri + Rust

Aplikasi desktop ringan untuk membuat evidence screenshot per test case.

## Fitur

- Pilih parent folder saja.
- Auto create folder `TC001`, `TC002`, `TC003`, dst.
- Screenshot manual via tombol atau shortcut.
- Dual-window screenshot untuk dua window berdampingan dalam satu layar tanpa taskbar bawah.
- Long screenshot halaman browser: scroll otomatis, deteksi overlap, lalu gabungkan menjadi satu PNG.
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
- Windows/Linux: `Ctrl + Shift + M` = Capture + Mark
- Windows/Linux: `Ctrl + Shift + D` = Dual Window Capture
- Windows: `Ctrl + Shift + A` = Long Capture
- Windows/Linux: `Ctrl + Shift + N` = Next TC
- Windows/Linux: `Ctrl + Shift + P` = Mark Pending
- Windows/Linux: `Ctrl + Shift + F` = Finish

- Mac: `Command + Shift + S` = Capture
- Mac: `Command + Shift + M` = Capture + Mark
- Mac: `Command + Shift + D` = Dual Window Capture
- Mac: `Command + Shift + A` = Long Capture
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

Mode Dual Window mengambil layar utama, memakai work area OS di Windows agar taskbar tidak ikut, lalu membuang strip atas default. Kalau hasil dual-window masih terlalu banyak/sedikit memotong bagian atas, ubah konstanta:

```rust
const DUAL_WINDOW_TOP_EDGE_IGNORE: u32 = 36;
```

## Catatan long screenshot

- Long Capture didukung di Windows dan macOS untuk halaman browser dengan scroll vertikal utama.
- Proses dimulai dari bagian atas halaman dan berhenti otomatis ketika viewport tidak berubah lagi.
- Jalankan shortcut ketika browser target aktif dan pointer berada di area halaman.
- Jangan scroll, mengetik, memindahkan window, atau menggerakkan fokus selama proses berlangsung.
- macOS memerlukan izin **Accessibility** dan **Screen Recording** untuk MyTBC.
- Sticky header sederhana ditangani saat stitching. Halaman dengan animasi terus-menerus, infinite scroll, atau nested scroll dapat dihentikan jika overlap tidak aman.


## Update 1.0.2 - Compact Action Layout

- Action button dibuat compact dalam grid 3 kolom agar tidak terlalu panjang ke bawah.
- Tombol Start kini berubah sesuai kondisi:
  - `Start Flow` saat belum mulai.
  - `Recording Aktif` saat flow sedang berjalan dan tombol dikunci agar tidak ambigu.
  - `Start TC Baru` saat flow sudah selesai.
  - `Resume Flow` saat session lama ditemukan.
- Ditambahkan kartu `Flow Status` agar user tahu jelas apakah aplikasi masih idle, ready, recording, atau finished.
## Catatan Browse Folder

Tombol **Browse** memakai dialog folder dari API frontend Tauri, bukan command blocking Rust. Ini dibuat supaya aplikasi tidak terasa freeze/lag saat native folder picker dibuka, terutama di Windows dan macOS tertentu.


## Update 1.0.5 - Check Update via GitHub Releases

- Ditambahkan section **Aplikasi & Update** di dalam aplikasi.
- Aplikasi menampilkan versi dari konfigurasi Tauri.
- Tombol **Check Update** akan mengecek release terbaru dari repo public GitHub:

```text
https://github.com/agungandhikaf/ss-record-rust/releases/latest
```

- Jika versi terbaru tersedia, tombol **Download Update** akan muncul dan membuka installer terbaru sesuai OS:
  - Windows: `MyScreenshots_v1.0.7_windows_x64_setup.exe`
  - Mac: `MyScreenshots_v1.0.7_mac.dmg`

Catatan: fitur ini tidak auto-install. User tetap perlu menutup aplikasi, menjalankan installer baru, lalu membuka aplikasi kembali.

## Update v1.0.7

- Bugfix session setelah aplikasi di-update: aplikasi menyimpan parent folder terakhir di app config dan otomatis membaca `session.json` dari folder tersebut saat dibuka lagi.
- Bugfix kompatibilitas session lama: aplikasi mencoba membaca beberapa key lama seperti `currentTC`/`currentTc` agar session dari versi sebelumnya tidak hilang.
- Penyesuaian crop macOS: crop bagian atas dinaikkan agar tab Chrome tidak ikut dan hasil lebih dekat ke area address/search bar.


## Update v1.0.7 - Rename aplikasi dan nama file release

- Nama aplikasi diubah dari **Flow Screenshot Recorder** menjadi **MyScreenshots**.
- Product name, title window, notifikasi, package name, dan bundle identifier sudah disesuaikan ke MyScreenshots.
- File installer di GitHub Actions/GitHub Releases sekarang diberi versi agar user tahu versi yang sedang didownload.

Contoh asset release:

```text
MyScreenshots_v1.0.7_windows_x64_setup.exe
MyScreenshots_v1.0.7_mac.dmg
```

Catatan migrasi: karena nama aplikasi dan identifier berubah, Windows/macOS bisa menganggap **MyScreenshots** sebagai aplikasi baru. Jika masih ada aplikasi lama **Flow Screenshot Recorder**, uninstall/hapus aplikasi lama satu kali agar user tidak bingung. Data screenshot di parent folder tetap aman.

## Pembaruan: Capture + Mark dan captions.txt (2026-10-27)

- Shortcut baru **Ctrl+Shift+M** (Windows/Linux) / **Command+Shift+M** (macOS) mengambil screenshot area browser aktif ke editor terpisah. Shortcut dan mode capture lama tidak diubah.
- Dalam editor, drag untuk menggambar kotak outline merah (tebal 3 px), ulangi untuk lebih dari satu kotak. **Ctrl/Command+Z** undo, **Ctrl/Command+Shift+Z** redo, **Enter** menyimpan, **Esc** membatalkan tanpa menambah evidence maupun metadata.
- Hasil mark disimpan dengan suffix `_mark.png` ke TC aktif, masuk ke `metadata.json`/`session.json`, dan dicoba disalin ke clipboard sebagaimana capture biasa.
- `captions.txt` dibuat saat TC dibuat serta diselaraskan ketika session dibaca, TC di-resume, dan setiap screenshot berhasil disimpan. Format: `1.`, baris kosong, `2.`, dst, mengikuti jumlah PNG. Nomor yang belum ada ditambahkan; **isi keterangan yang sudah diketik pengguna tidak ditimpa dan tidak dihapus**. Jika PNG dihapus manual, nomor lama dibiarkan agar catatan pengguna tidak hilang; edit manual jika penomoran perlu dirapikan.
- File yang berubah: `index.html`, `src/main.js`, `src/styles.css`, `src-tauri/src/main.rs`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `README.md`.
- Verifikasi yang diperlukan di macOS/Windows: compile `npm run dev`, izin Screen Recording macOS, shortcut global ketika browser fokus, gambar rectangle/undo/redo, Enter/Esc, clipboard, count dan isi captions untuk semua mode capture, serta regresi flow dan long capture.


## Pembaruan lanjutan: auto-focus editor mark + refresh icon aplikasi (2026-10-27)

- **Fix UX Capture + Mark**: saat shortcut **Ctrl/Command + Shift + M** ditekan dari aplikasi lain, MyTBC kini **mengambil screenshot lebih dulu**, lalu otomatis memunculkan window aplikasi ke depan dan fokus ke editor mark. Dengan alur ini, screenshot target tidak tertangkap bersama window MyTBC, dan user **tidak perlu klik aplikasi dulu** untuk mulai menggambar kotak merah.
- Saat editor mark ditutup lewat **Enter** atau **Esc**, mode `always on top` pada window dilepas kembali agar perilaku window normal.
- **Refresh icon aplikasi**: aset icon bundle diganti agar tampilan MyTBC lebih modern dan lebih selaras dengan fungsi screenshot evidence + rectangle marking. File icon yang diperbarui: `src-tauri/icons/32x32.png`, `src-tauri/icons/128x128.png`, `src-tauri/icons/128x128@2x.png`, `src-tauri/icons/icon.ico`, `src-tauri/icons/icon.icns`.
- File kode yang berubah untuk fixing ini: `src/main.js`, `src-tauri/src/main.rs`, `README.md`, dan seluruh file icon di `src-tauri/icons/`.
- Verifikasi yang disarankan: tekan shortcut Capture + Mark saat browser aktif (tanpa klik MyTBC), pastikan editor langsung tampil fokus; uji **Esc**, **Enter**, **Undo/Redo**, clipboard, penomoran screenshot, serta pastikan icon baru muncul saat aplikasi di-run/build ulang.

## Redesign V2 — Light/Dark Mode (2026-10-28)

- Tampilan frontend diubah menjadi dashboard desktop dengan sidebar: **Overview, Test Cases, Evidence, Settings, Update**.
- **Overview**: quick-capture Screenshot, Capture + Mark, Dual Window, Long Screenshot, Fullscreen; kartu Flow Aktif; Recent Evidence; rangkuman test case.
- Warna berdasarkan fungsi: biru untuk capture/action utama, hijau untuk recording, oranye untuk pending, dan merah untuk finish. Background/panel didesain netral agar nyaman untuk penggunaan panjang.
- **Light dan Dark mode** dapat diganti dari header atau Settings. Preferensi disimpan secara lokal di `localStorage` (`mytbc-theme-v2`) dan dipulihkan saat aplikasi dibuka kembali.
- Aksi/ID backend dan shortcut tetap menggunakan mekanisme lama; **tidak ada perubahan Rust, format screenshot, metadata, session, captions, atau alur capture**.
- Menu Evidence menampilkan capture terakhir dari state sesi saat ini dan tombol membuka folder TC/parent. Ini bukan galeri seluruh PNG; untuk seluruh file gunakan Open Current TC.
- File yang berubah: `index.html`, `src/main.js`, `src/styles.css`, `README.md`. Update ini dikemas sebagai ZIP patch berisi file berubah saja (path relatif dari root project).
- Pengujian: `npm ci && npm run dev`, cek perpindahan menu, persistensi light/dark setelah restart, shortcut global saat browser fokus, flow start/next/pending/finish/resume, semua mode capture, mark editor, captions.txt, serta halaman Update.
