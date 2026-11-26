# AI Vibe Coding Rules

## Purpose
Aturan kolaborasi AI coding untuk menjaga codebase konsisten dan mencegah scope creep.

## Workflow
1. Inspect project dan code relevan.
2. Pahami architecture, Bevy version, dependencies.
3. Ringkas perubahan yang direncanakan dan asumsi.
4. Implementasikan task yang disepakati saja.
5. Hindari refactor unrelated.
6. Jalankan `cargo check` dan test relevan.
7. Laporkan hasil dan issue.
8. Usulkan next task; jangan langsung mengimplementasikannya.

## Constraints
- Jangan rewrite project tanpa kebutuhan.
- Jangan ubah dependency version tanpa persetujuan.
- Jangan tambah dependency tanpa menjelaskan dan meminta persetujuan.
- Jangan invent fitur yang belum disetujui.
- Module kecil dan maintainable.
- Pisahkan logic dari visual.
- Comment untuk logic non-obvious.
- Ikuti style code existing.
- Jangan mengklaim test berhasil jika belum dijalankan.

## Completion report
Laporkan: implementasi, file yang berubah, keputusan, commands/tests dan hasil, issues/limitations, next task (belum dieksekusi).

## Checkpoint
Git checkpoint/commit pada milestone yang disepakati. Commit message deskriptif.
