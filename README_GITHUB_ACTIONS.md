# Build `.exe` dan `.dmg` Pakai GitHub Actions

Ini solusi yang disarankan kalau:

- Laptop kamu Mac, tapi butuh build Windows `.exe`.
- PC kantor tidak bisa install dependency karena jaringan kantor memblokir npm/cargo.
- Kamu tidak mau kirim folder project yang besar.

## Langkah

### 1. Buat repo GitHub private

```text
GitHub > New Repository > Private
Nama: flow-screenshot-recorder-tauri
```

### 2. Upload source project

Upload semua file project ini, tapi jangan upload folder berikut:

```text
node_modules/
dist/
src-tauri/target/
```

### 3. Jalankan Actions

Masuk ke repo:

```text
Actions > Build Tauri Installers > Run workflow
```

### 4. Download hasil build

Setelah workflow selesai, buka run terakhir lalu download artifact:

```text
flow-screenshot-recorder-windows
flow-screenshot-recorder-macos
```

Di dalamnya akan ada installer:

```text
Windows: .exe
Mac: .dmg
```

File itulah yang dibagikan ke user akhir.

## User akhir tidak perlu install

User akhir tidak perlu:

```text
Node.js
Rust
npm install
cargo build
```

User akhir cukup install file `.exe` atau `.dmg` hasil artifact GitHub Actions.
