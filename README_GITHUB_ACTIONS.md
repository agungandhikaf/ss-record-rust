# Build Tauri Installer via GitHub Actions

Pastikan repo berisi folder:

```text
src-tauri/
```

Kalau tidak ada folder `src-tauri`, berarti itu bukan project Tauri.

## Langkah

1. Upload source project ini ke GitHub.
2. Jangan upload folder besar berikut:

```text
node_modules/
dist/
src-tauri/target/
```

3. Pastikan file ini ada:

```text
.github/workflows/build-installers.yml
```

4. Buka tab GitHub `Actions`.
5. Pilih `Build Tauri Installers`.
6. Klik `Run workflow`.
7. Tunggu sampai success.
8. Download artifact:

```text
tauri-windows-installer
tauri-mac-installer
```

## Output

Windows artifact biasanya berisi file `.exe` dari folder:

```text
src-tauri/target/release/bundle/nsis/
```

Mac artifact biasanya berisi file `.dmg` dari folder:

```text
src-tauri/target/release/bundle/dmg/
```
