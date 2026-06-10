# Panduan Update Version MyTBC

Panduan ini dipakai setiap kali menaikkan versi aplikasi MyTBC sebelum build installer dan membuat GitHub Release.

## Prinsip Utama

Jangan pernah memakai **Find & Replace global** untuk mengganti nomor versi. Find & Replace global bisa ikut mengubah versi dependency di `package-lock.json` atau `Cargo.lock`, misalnya dependency npm berubah menjadi versi aplikasi dan menyebabkan build gagal.

Contoh kasus yang harus dihindari:

```text
@types/estree 1.0.9 ikut berubah menjadi @types/estree 1.0.10
```

Kalau dependency ikut berubah, GitHub Actions bisa gagal di step `npm install` dengan error `404 Not Found`.

## File yang Perlu Diubah

Untuk menaikkan versi, hanya bagian version aplikasi yang boleh berubah di file berikut:

```text
package.json
package-lock.json
src-tauri/tauri.conf.json
src-tauri/Cargo.toml
src-tauri/Cargo.lock
```

Catatan:

- `package-lock.json` sebaiknya diubah otomatis lewat `npm version`, bukan diedit manual.
- `Cargo.lock` sebaiknya berubah otomatis lewat cargo. Kalau tidak ada perubahan dependency Rust, cukup pastikan package root `mytbc-tauri` memakai versi yang sama.
- Jangan mengubah versi dependency seperti `@types/estree`, `vite`, `tauri`, `serde`, `image`, dan dependency lain kecuali memang sedang update dependency.

## Cara Aman Update Version

Misalnya versi baru adalah `1.0.10`.

### 1. Update versi npm

Jalankan dari root project:

```bash
npm version 1.0.10 --no-git-tag-version
```

Command ini akan mengubah versi aplikasi di:

```text
package.json
package-lock.json
```

### 2. Update versi Tauri config

Edit file:

```text
src-tauri/tauri.conf.json
```

Ubah hanya bagian ini:

```json
"package": {
  "productName": "MyTBC",
  "version": "1.0.10"
}
```

### 3. Update versi Cargo

Edit file:

```text
src-tauri/Cargo.toml
```

Ubah hanya bagian `[package]`:

```toml
[package]
name = "mytbc-tauri"
version = "1.0.10"
```

### 4. Update atau validasi Cargo.lock

Kalau di laptop ada Rust/Cargo, jalankan:

```bash
cd src-tauri
cargo check
cd ..
```

Kalau `Cargo.lock` berubah otomatis, ikut commit file tersebut.

Kalau hanya version root package yang perlu disesuaikan, pastikan di `src-tauri/Cargo.lock` bagian ini sama:

```toml
[[package]]
name = "mytbc-tauri"
version = "1.0.10"
```

## Cek Sebelum Commit

Jalankan:

```bash
git diff
```

Pastikan yang berubah hanya versi aplikasi dan file fitur yang memang sedang dikerjakan.

Cek khusus npm dependency:

```bash
grep -n "node_modules/@types/estree" -A4 package-lock.json
```

Pastikan versinya tetap versi dependency asli, bukan ikut versi aplikasi.

Contoh aman:

```text
"version": "1.0.9"
"resolved": "https://registry.npmjs.org/@types/estree/-/estree-1.0.9.tgz"
```

Contoh bahaya:

```text
"version": "1.0.10"
"resolved": "https://registry.npmjs.org/@types/estree/-/estree-1.0.10.tgz"
```

Kalau contoh bahaya muncul, restore `package-lock.json`, lalu ulangi dengan `npm version`.

## Test Build Lokal

Minimal jalankan:

```bash
npm install
npm run build:vite
```

Untuk test Tauri dev:

```bash
npm run dev
```

Untuk build lokal Windows:

```bash
npm run dist:win
```

Untuk build lokal Mac:

```bash
npm run dist:mac
```

## Commit dan Tag Release

Setelah semua aman:

```bash
git add package.json package-lock.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock UPDATE_VERSION_GUIDE.md
git commit -m "chore: release v1.0.10"
git push
```

Buat tag:

```bash
git tag v1.0.10
git push origin v1.0.10
```

GitHub Actions akan build installer dan upload ke GitHub Release.

Expected nama file release:

```text
MyTBC_x64_v.1.0.10.exe
MyTBC_mac_v.1.0.10.dmg
```

## Kalau Terlanjur Salah Find & Replace

Restore lock file dari commit sebelumnya:

```bash
git checkout HEAD~1 -- package-lock.json
```

Lalu jalankan ulang cara aman:

```bash
npm version 1.0.10 --no-git-tag-version
```

Cek ulang dengan:

```bash
git diff
```
