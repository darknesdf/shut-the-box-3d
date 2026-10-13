# Shut The Box 3D (Rust)

Penulisan ulang dari versi Godot/GDScript ke **Rust + Bevy 0.15**. Aturan main dan animasi
sama dengan versi aslinya:

- Dadu 3D dengan mata dadu yang digambar prosedural, dilempar dengan lengkung + pantulan teredam
- Nampan kayu 3D, kartu 1-9 yang naik saat dipilih dan terlipat dengan pantulan kecil
- Kalah: kartu sisa memerah dan lunglai, kamera bergetar, kilatan merah, panel "YOU LOST"
- Menang: confetti + panel "BOX SHUT!"
- Tombol Clear, statistik best / wins / games tersimpan lokal
- Kontrol sentuh / mouse, layout landscape

## Cara main

1. Tekan **ROLL** (jika kartu 7, 8, 9 sudah tertutup, hanya 1 dadu yang dilempar).
2. Pilih kartu yang jumlahnya sama dengan hasil lemparan; otomatis terlipat jika pas.
3. Tutup semua 9 kartu untuk menang. Jika tak ada kombinasi, skor = jumlah kartu yang masih terbuka (makin kecil makin bagus).

## Jalankan di desktop

```bash
cargo run --release
cargo test --lib        # tes aturan permainan, tween, picking, save
```

Linux butuh: `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev`.

## Build APK Android

```bash
rustup target add aarch64-linux-android
cargo install cargo-apk
# pastikan ANDROID_HOME dan ANDROID_NDK_ROOT sudah diset, dan keystore debug ada di ~/.android/debug.keystore
cargo apk build --release --lib
```

Atau push ke GitHub: workflow `Build` menghasilkan artifact `ShutTheBox3D-apk`.
Ganti package name di `[package.metadata.android]` pada `Cargo.toml` (dan path simpan di `src/save.rs`).

## Struktur

| File | Isi |
| --- | --- |
| `src/logic.rs` | aturan murni (subset-sum, toggle, satu/dua dadu) + unit test |
| `src/game.rs` | state machine permainan (pengganti `main.gd`) |
| `src/tile.rs` | kartu 1-9, animasi pilih / lipat / kalah, tekstur angka 7-segmen |
| `src/die.rs` | dadu 3D + animasi lempar |
| `src/anim.rs` | tween kecil (`Channel`) pengganti `Tween` Godot |
| `src/input.rs` | picking sentuh/klik lewat ray vs kotak |
| `src/world.rs` | kamera, lampu, nampan |
| `src/ui.rs` | HUD, tombol, overlay menang/kalah |
| `src/confetti.rs` | confetti |
| `src/save.rs` | statistik tersimpan |
