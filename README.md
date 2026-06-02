# Flow Screenshot Recorder - Tauri + Rust

Versi ringan dari aplikasi **Flow Screenshot Recorder** menggunakan **Tauri + Rust**.

Aplikasi ini dibuat dengan konsep yang sama seperti versi Electron sebelumnya:

- User pilih **Parent Folder** saja.
- Aplikasi otomatis membuat folder `TC001`, `TC002`, `TC003`, dan seterusnya.
- Screenshot dilakukan manual lewat tombol atau shortcut.
- Selama belum klik **Next TC**, semua screenshot masuk ke TC aktif.
- Nama file otomatis unik agar tidak duplicate.
- Tidak ada popup window setelah screenshot via shortcut.

---

## Struktur Output

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

---

## Shortcut

| Aksi | Windows | Mac |
|---|---|---|
| Capture Screenshot | Ctrl + Shift + S | Command + Shift + S |
| Next TC | Ctrl + Shift + N | Command + Shift + N |
| Finish Flow | Ctrl + Shift + F | Command + Shift + F |

Kalau shortcut bentrok dengan aplikasi lain, tombol manual di UI tetap bisa dipakai.

---

## Cara Run di Mac

Syarat satu kali:

1. Install Node.js LTS: https://nodejs.org/
2. Install Rust: https://rustup.rs/

Lalu:

```text
Double click RUN_MAC.command
```

Kalau Mac menolak file `.command`, klik kanan file tersebut lalu pilih **Open**.

---

## Cara Build `.dmg` di Mac

```text
Double click BUILD_INSTALLER_MAC.command
```

Hasil build ada di:

```text
src-tauri/target/release/bundle/dmg/
```

---

## Cara Run di Windows

Syarat satu kali:

1. Install Node.js LTS: https://nodejs.org/
2. Install Rust: https://rustup.rs/

Lalu:

```text
Double click RUN_WINDOWS.bat
```

---

## Cara Build `.exe` di Windows

```text
Double click BUILD_INSTALLER_WINDOWS.bat
```

Hasil build ada di:

```text
src-tauri/target/release/bundle/nsis/
```

---

## Build Windows dari Mac

Laptop Mac tidak ideal untuk build `.exe` Windows secara lokal.

Solusi paling aman: pakai **GitHub Actions** yang sudah disediakan di file:

```text
.github/workflows/build-installers.yml
```

Alurnya:

```text
1. Upload source project ini ke GitHub private repo
2. Buka tab Actions
3. Jalankan workflow “Build Tauri Installers”
4. Download artifact Windows .exe dan Mac .dmg
```

Dengan cara ini, user kantor tidak perlu install dependency dan kamu tidak perlu build Windows dari Mac.

---

## Catatan macOS Screen Recording

Kalau screenshot kosong/hitam di Mac, aktifkan permission:

```text
System Settings
Privacy & Security
Screen Recording
Aktifkan Flow Screenshot Recorder / Terminal
```

Setelah mengaktifkan permission, tutup dan buka ulang aplikasi.

---

## Catatan Ukuran File

Tauri biasanya jauh lebih kecil dari Electron karena tidak membawa Chromium penuh. Hasil installer bisa bervariasi tergantung OS dan dependency, tapi umumnya lebih ringan dibanding versi Electron.

---

## Batasan MVP

- Capture masih mengambil layar pertama/primary monitor.
- Belum ada selector multi-monitor.
- Belum ada export Word/PDF.
- Belum ada floating button.
- Belum ada auto capture setelah klik mouse.
