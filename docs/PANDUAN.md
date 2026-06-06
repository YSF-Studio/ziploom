# Panduan Pengguna ZipLoom

ZipLoom adalah utilitas arsip desktop **100% offline** untuk mengompres, mengekstrak, dan memeriksa isi arsip — dibangun dengan Tauri, Rust, dan Svelte.

> **Penting:** Hasil analisis forensik bersifat informatif. Verifikasi sendiri sebelum dipakai untuk keperluan hukum atau audit resmi.

---

## Daftar isi

1. [Instalasi](#instalasi)
2. [Menjalankan aplikasi](#menjalankan-aplikasi)
3. [Antarmuka umum](#antarmuka-umum)
4. [Tab Compress](#tab-compress)
5. [Tab Extract](#tab-extract)
6. [Tab Inspect](#tab-inspect)
7. [Tab About](#tab-about)
8. [Arsip berpassword](#arsip-berpassword)
9. [Drag & drop](#drag--drop)
10. [Dukungan format](#dukungan-format)
11. [Pemecahan masalah](#pemecahan-masalah)

---

## Instalasi

### Opsi A — Unduh rilis siap pakai

1. Buka [Releases](https://github.com/YSF-Studio/ziploom/releases).
2. Unduh installer untuk sistem operasi Anda (macOS / Windows / Linux).
3. Pasang sesuai petunjuk OS.

### Opsi B — Bangun dari sumber

**Prasyarat**

| Platform | Dependensi utama |
|----------|------------------|
| **macOS** | Xcode Command Line Tools, Node.js 22+, Rust stable |
| **Linux** | `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`, `libssl-dev`, `libpcap-dev` |
| **Windows** | Visual Studio Build Tools (C++), Node.js 22+, Rust stable |

```bash
git clone https://github.com/YSF-Studio/ziploom.git
cd ziploom
npm install
npm run tauri:dev      # mode pengembangan
npm run tauri:build    # menghasilkan installer
```

> Jangan jalankan `npm run dev` saja — fitur arsip membutuhkan aplikasi Tauri (`npm run tauri:dev`).

---

## Menjalankan aplikasi

```bash
npm run tauri:dev
```

Server pengembangan Vite: `http://localhost:1422` (hanya untuk debug UI; fitur arsip tetap lewat proses Tauri).

---

## Antarmuka umum

| Area | Fungsi |
|------|--------|
| **Traffic lights** (kiri atas) | Tutup / minimalkan / maksimalkan jendela |
| **Tab bar** | Compress · Extract · Inspect · About |
| **Toggle tema** (kanan atas) | Klik untuk berganti **Light mode** → **Dark mode** → **System default** |
| **Status bar** (bawah) | Status proses & badge **Offline** |
| **Toast** | Notifikasi sukses / error singkat |

Tidak ada tab Settings terpisah — pengaturan tema langsung dari tombol berlabel di titlebar.

---

## Tab Compress

Kompres file dan folder ke berbagai format arsip.

### Langkah

1. Buka tab **Compress**.
2. Tambahkan sumber:
   - **Browse files** — pilih satu atau banyak file
   - **Browse folder** — pilih folder utuh
   - **Drag & drop** file/folder ke dropzone
3. Atur opsi:
   - **Format** — ZIP, TAR, TAR.GZ, TAR.BZ2, TAR.XZ, TAR.ZST
   - **Password** — hanya untuk **ZIP** (AES-256, kompatibel 7-Zip / WinRAR)
   - **Clean macOS metadata** — abaikan `.DS_Store` dan `__MACOSX`
   - **Slider Compress** — tingkat kompresi (Fast → Best)
4. Klik **Compress**.
5. Pilih lokasi dan nama file arsip di dialog **Simpan**.
6. Hasil ditampilkan di bawah tombol (jumlah file & path output).

### Tips

- Password **hanya** didukung untuk format ZIP.
- Beberapa file macOS otomatis difilter jika opsi clean metadata aktif.
- Chip sumber bisa dihapus dengan tombol **×**.

---

## Tab Extract

Ekstrak isi arsip ke folder pilihan Anda.

### Langkah

1. Buka tab **Extract**.
2. Pilih arsip:
   - **Choose archive** / klik dropzone
   - Drag & drop file arsip
3. (Opsional) centang **Remove __MACOSX/ and .DS_Store**.
4. Klik **Extract**.
5. Pilih folder tujuan.
6. Jika arsip berpassword, dialog password akan muncul.

### Format yang didukung

ZIP, TAR, TAR.GZ, TAR.BZ2, TAR.XZ, TAR.ZST, 7z, RAR *(lihat [catatan platform](#dukungan-format))*.

---

## Tab Inspect

Periksa isi arsip tanpa mengekstrak semuanya — cocok untuk audit cepat dan analisis forensik ringan.

### Alur kerja

```
Pilih arsip → Load metadata → (opsional) Full Scan → Preview / Export / Extract terpilih
```

### Langkah dasar

1. Buka tab **Inspect**.
2. Pilih arsip (browse atau drag & drop).
3. Klik **Load** — memuat daftar file, ukuran, dan metadata dasar.
4. Untuk arsip berpassword, masukkan password saat diminta.

### Aksi lanjutan

| Tombol | Fungsi |
|--------|--------|
| **Full Scan** | Hash per file (MD5/SHA1/SHA256), entropy, magic-byte, deteksi ancaman/anomali |
| **Hash All** | Hash file arsip (container) secara keseluruhan |
| **Export CSV** | Ekspor laporan ke CSV |
| **Extract Selected** | Ekstrak hanya file yang dicentang |

### Panel & filter

- **Tree / Flat** — tampilan hierarki atau datar
- **Search** — filter nama path
- **Flagged only** — hanya entri yang ditandai
- **Columns ▾** — tampilkan/sembunyikan Hash, Entropy, Magic, Modified
- **Detail panel** — preview teks/hex/gambar (dibatasi ukuran), ringkasan risiko, tab threats/anomalies

### Preview file

Klik baris file di tabel untuk memuat preview di panel kanan (read-only, tidak mengeksekusi file).

---

## Tab About

Informasi aplikasi, daftar fitur, disclaimer hukum, dan tautan [ysfloom.com](https://ysfloom.com).

---

## Arsip berpassword

| Operasi | ZIP berpassword |
|---------|-----------------|
| **Compress** | ✅ Aktifkan checkbox Password, isi kata sandi, format ZIP |
| **Extract** | ✅ Dialog password otomatis |
| **Inspect** | ✅ Password diminta saat Load / Full Scan |

Format ZIP menggunakan enkripsi **AES-256** standar — dapat dibuka di 7-Zip, WinRAR, dan ZipLoom.

> Enkripsi file tunggal `.aes256` (bukan ZIP) masih tersedia di backend untuk keperluan internal/uji, tetapi **tidak** ditampilkan di UI utama.

---

## Drag & drop

| Tab aktif | Perilaku drop |
|-----------|---------------|
| **Compress** | Menambahkan file/folder ke antrian sumber |
| **Extract** | Mengisi path arsip pertama yang di-drop |
| **Inspect** | Memuat arsip yang di-drop untuk inspeksi |

---

## Dukungan format

| Format | Compress | Extract | Inspect |
|--------|:--------:|:-------:|:-------:|
| ZIP | ✅ | ✅ | ✅ |
| ZIP + password | ✅ | ✅ | ✅ |
| TAR | ✅ | ✅ | ✅ |
| TAR.GZ / .tgz | ✅ | ✅ | ✅ |
| TAR.BZ2 / .tbz2 | ✅ | ✅ | ✅ |
| TAR.XZ / .txz | ✅ | ✅ | ✅ |
| TAR.ZST / .tzst | ✅ | ✅ | ✅ |
| 7z | — | ✅ | ✅ |
| RAR | — | ✅* | ✅* |

\* **RAR tidak didukung di Windows** (keterbatasan build native `unrar`). macOS dan Linux mendukung ekstraksi & inspeksi RAR.

---

## Pemecahan masalah

| Gejala | Solusi |
|--------|--------|
| `invoke` / fitur tidak jalan di browser | Jalankan `npm run tauri:dev`, bukan `npm run dev` |
| Password ditolak | Pastikan format ZIP; periksa huruf besar/kecil password |
| RAR gagal di Windows | Gunakan 7z/ZIP, atau ekstrak di macOS/Linux |
| Full Scan lambat | Normal pada arsip besar; pantau progress bar |
| Build Linux gagal | Pasang `libwebkit2gtk-4.1-dev` dan dependensi GTK (lihat README) |

### File contoh untuk uji manual

- `samples/` — dokumen demo
- `tests/fixtures/e2e/` — fixture otomatis (`sample_alpha.txt`, `nested/sample_gamma.txt`, dll.)

---

## Privasi

ZipLoom **tidak mengirim data ke internet**. Tidak ada telemetri, analitik, atau panggilan jaringan untuk fitur inti.

---

**© 2026 YSF Studio** · [GitHub](https://github.com/YSF-Studio/ziploom) · [ysfloom.com](https://ysfloom.com)
