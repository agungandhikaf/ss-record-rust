# Build dan Release Tauri Installer via GitHub Actions

Pastikan repo berisi folder:

```text
src-tauri/
```

Kalau tidak ada folder `src-tauri`, berarti itu bukan project Tauri.

## Build biasa untuk testing

1. Push perubahan ke branch `main`.
2. Buka tab GitHub `Actions`.
3. Pilih `Build Tauri Installers`.
4. Tunggu sampai success.
5. Download artifact:

```text
tauri-windows-installer
tauri-mac-installer
```

Artifact ini cocok untuk testing developer.

## Release resmi untuk user

Untuk membuat release yang bisa didownload user dari menu **Check Update**, buat dan push tag versi.

Contoh release `v1.0.5`:

```bash
git add .
git commit -m "release: v1.0.5"
git push origin main

git tag v1.0.5
git push origin v1.0.5
```

Saat tag `v1.0.5` dipush, GitHub Actions akan:

```text
1. Build Windows installer
2. Build Mac installer
3. Upload artifact untuk developer
4. Publish installer ke GitHub Releases
```

Asset release dibuat dengan nama stabil:

```text
Flow_Screenshot_Recorder_windows_x64_setup.exe
Flow_Screenshot_Recorder_mac.dmg
```

Nama stabil ini penting supaya tombol **Download Update** di aplikasi bisa mengambil installer terbaru dari GitHub Releases.

## Hal yang wajib dicek sebelum release

Pastikan version sama di dua file:

```text
package.json
src-tauri/tauri.conf.json
```

Contoh:

```json
"version": "1.0.5"
```

Tag harus mengikuti version:

```text
v1.0.5
```

## Link user

User bisa download dari:

```text
https://github.com/agungandhikaf/ss-record-rust/releases/latest
```

Atau lewat tombol **Check Update** di aplikasi.

## Catatan v1.0.6

Jika user melaporkan session tidak terbaca setelah update, pastikan mereka pernah memilih parent folder minimal satu kali pada versi terbaru. Setelah itu aplikasi akan menyimpan parent folder terakhir dan auto-restore session pada pembukaan berikutnya.

