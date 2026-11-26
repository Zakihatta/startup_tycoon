# Startup Tycoon — Asset Preparation & Visual Direction

> Dokumen ini menjadi panduan persiapan visual, aset, format file, pipeline, dan daftar aset sebelum masuk ke milestone pengembangan awal (M1) menggunakan Rust + Bevy.

---

## 1. Ringkasan Proyek

**Working title:** Startup Tycoon  
**Genre:** Startup / Company Management / Tycoon / Sandbox Simulation  
**Engine:** Bevy  
**Programming language:** Rust  
**Perspective:** First-person, 3D  
**Setting awal:** Perusahaan software kecil dengan kantor sewaan sederhana.

### Konsep

Pemain berperan sebagai pendiri startup teknologi. Pemain mengembangkan produk, mengelola karyawan, keuangan, pasar, dan kantor. Sistem manajemen disajikan melalui UI, sementara kantor menjadi ruang 3D yang bisa dijelajahi dan diinteraksikan pemain.

### Core pillars

- **Build:** membangun produk, workstation, dan kantor.
- **Manage:** mengatur pegawai, uang, dan aktivitas perusahaan.
- **Simulate:** waktu, produktivitas, pengembangan, pasar, pendapatan, dan kejadian.
- **Expand:** meningkatkan fasilitas dan skala perusahaan.
- **Experience:** menjalani simulasi melalui eksplorasi first-person.

### Gameplay loop

`Plan → Build → Operate → Analyze → Improve / Expand`

---

## 2. Visual Direction

### Style yang disarankan

Gunakan **stylized modern 3D**: bentuk sederhana, proporsi sedikit stylized tetapi tidak terlalu kartun, material bersih, detail secukupnya, pencahayaan lembut, dan ruang kerja modern.

Tujuan visual adalah membuat aset konsisten, mudah dibaca, dan relatif ringan untuk dikembangkan oleh tim kecil atau solo developer.

### Prinsip desain

- Minimalis dan rapi.
- Modern, hangat, dan terasa seperti ruang startup.
- Hindari detail realistis yang terlalu mahal dibuat.
- Gunakan palet material dan warna yang konsisten.
- Proporsi objek konsisten.
- Aset modular sebisa mungkin.
- Prioritaskan siluet yang mudah dikenali.
- UI harus mudah dibaca dan tidak memenuhi layar.

### Palet awal (tentatif)

| Kategori | Arahan |
|---|---|
| Lingkungan | Putih hangat, abu-abu muda, beige |
| Furniture | Kayu natural, putih, abu-abu, hitam |
| Accent | Hijau, biru, atau warna branding perusahaan |
| UI | Dark/light neutral, accent tunggal |
| Lighting | Natural daylight, warm indoor lighting |

> Palet ini bersifat sementara. Pilih palet final setelah satu ruangan prototype dibuat.

---

## 3. Prinsip Asset Pipeline

### Asset runtime vs source

| Asset | Format runtime yang disarankan | Source/editable |
|---|---|---|
| Character 3D | `.glb` | `.blend` |
| Furniture | `.glb` | `.blend` |
| Environment modular | `.glb` | `.blend` |
| Equipment | `.glb` | `.blend` |
| Animations | Dalam GLB jika memungkinkan | `.blend` / source terkait |
| UI icon | `.png` | `.svg` atau file desain |
| Portrait | `.png` | `.blend` atau source ilustrasi |
| Texture | `.png` / `.jpg` | Source sesuai pipeline |
| Audio | `.ogg` | `.wav` |
| Font | `.ttf` / `.otf` | File font berlisensi |

### Format model

**GLB** menjadi format praktis untuk aset 3D runtime karena dapat menyimpan mesh, material, texture, skeleton, dan animasi dalam satu file (jika aset mendukungnya).

**BLEND** adalah file kerja Blender, bukan file runtime yang harus dimuat game.

### Catatan lisensi

Sebelum memakai aset dari marketplace atau repositori:

- Periksa lisensi dan izin penggunaan komersial.
- Periksa atribusi yang diwajibkan.
- Periksa apakah aset boleh dimodifikasi.
- Simpan sumber, nama creator, URL, lisensi, dan tanggal pengunduhan.
- Jangan berasumsi aset gratis otomatis bebas digunakan untuk semua tujuan.

---

## 4. Karakter

### 4.1 Player / Founder

Player adalah karakter founder yang digunakan dalam first-person game. Untuk MVP, model tubuh player mungkin tidak terlihat dalam gameplay (karena first-person), tetapi dapat berguna untuk shadow, third-person debugging, cutscene, atau fitur masa depan.

**Spesifikasi ideal:**

- Humanoid 3D.
- Rigged.
- Game-ready.
- Stylized modern.
- Skeleton yang dapat digunakan bersama atau disesuaikan untuk animasi humanoid.
- GLB untuk runtime.
- Blender source jika memungkinkan.

**Prioritas MVP:**

- [ ] Satu model founder/player (opsional untuk first-person prototype).
- [ ] Rig/skeleton bila model digunakan.
- [ ] Idle.
- [ ] Walk (bila karakter visual digunakan).

> Untuk M1 first-person, player capsule + camera dapat cukup. Jangan menghambat movement prototype karena belum memiliki model player.

### 4.2 Employee Character

Karyawan ditampilkan sebagai NPC di kantor. MVP hanya memerlukan satu role visual, yaitu Developer.

**Spesifikasi:**

- Humanoid stylized.
- Rigged.
- Game-ready.
- Memiliki pose idle.
- Memiliki animasi duduk/typing bila tersedia.
- Skeleton/rig konsisten dengan karakter lain bila memungkinkan.
- GLB.

**Animasi prioritas:**

| Animasi | Prioritas |
|---|---|
| Idle / standing | MVP |
| Walk | MVP jika NPC bergerak |
| Sitting | Sangat disarankan |
| Typing / working | Sangat disarankan |
| Talking | Future |
| Reacting / celebrating | Future |
| Sleep / exhausted | Future |

### 4.3 Modular character system (future)

Untuk variasi karyawan tanpa harus membuat banyak model:

- Base body.
- Hair variants.
- Outfit variants.
- Accessories.
- Skin/material variants.

Gunakan pipeline dan skeleton yang konsisten. Hindari mengumpulkan banyak model dari pack berbeda sebelum memastikan rig dan animasinya kompatibel.

### 4.4 Employee portrait

Portrait diperlukan untuk UI kandidat, employee list, profile, dan report.

**Format:** PNG.

Portrait bisa berupa:

- Render karakter 3D.
- Ilustrasi stylized.
- Crop dari character render.

Portrait bukan pengganti model NPC 3D di dunia.

---

## 5. Environment / Office

Bangun kantor dari komponen modular. Hindari mengimpor satu model kantor besar yang tidak dapat diubah.

### 5.1 Struktur dasar ruangan

**MVP:**

- [ ] Floor.
- [ ] Wall module.
- [ ] Corner wall (jika diperlukan).
- [ ] Door.
- [ ] Window.
- [ ] Ceiling (opsional, dapat ditunda).
- [ ] Lighting.
- [ ] Entry area.

### 5.2 Modular wall

Sebaiknya sediakan beberapa komponen:

- Wall segment lurus.
- Corner segment.
- Wall segment dengan pintu.
- Wall segment dengan jendela.
- Interior divider/partition (future).

Komponen modular memudahkan ekspansi, layout, dan building system.

### 5.3 Floor

- Floor tiles atau satu modul lantai yang dapat diulang.
- Dimensi grid konsisten.
- Material yang mudah diganti.
- Hindari texture terlalu ramai.

### 5.4 Door & window

- Door sebagai objek terpisah.
- Window sebagai objek terpisah.
- Door dapat diinteraksikan atau dianimasikan pada fase berikutnya.
- Window dapat menjadi visual/environment detail.

### 5.5 Lighting

Untuk prototype awal:

- Satu sumber daylight atau ambient setup.
- Beberapa lampu indoor.
- Hindari setup lighting yang kompleks sebelum performance dan visual dasar stabil.

---

## 6. Furniture

Furniture sebaiknya berupa aset terpisah dan modular.

### MVP

| Asset | Jumlah awal | Prioritas |
|---|---:|---|
| Desk | 1–2 | Must-have |
| Office chair | 1–2 | Must-have |
| Computer / desktop | 1 | Must-have |
| Monitor | 1 | Must-have |
| Keyboard | 1 | Nice-to-have |
| Mouse | 1 | Nice-to-have |
| Small cabinet | 0–1 | Optional |
| Trash bin | 0–1 | Optional |

### Future

- Sofa.
- Coffee table.
- Meeting table.
- Meeting chairs.
- Whiteboard.
- Bookshelf.
- Plants.
- Storage cabinet.
- Reception desk.
- Printer.
- Additional workstation.
- Server rack.
- Decoration.

### Asset requirements

- GLB.
- Pivot/origin yang masuk akal.
- Scale konsisten.
- Material sederhana.
- Mesh tidak terlalu kompleks.
- Model furniture individual, bukan satu set yang menyatu.

---

## 7. Equipment

Equipment memiliki dua fungsi: objek visual dan objek gameplay.

### MVP

- [ ] Desktop computer.
- [ ] Monitor.
- [ ] Desk/chair workstation.

### Future

- Laptop.
- Server.
- Networking equipment.
- Multiple monitors.
- Printer.
- Meeting display.
- Storage/backup equipment.

### Separation of concern

Statistik dan logic equipment (misalnya upgrade productivity, cost, maintenance) disimpan di data/game logic Rust. Model GLB hanya menangani tampilan dan animasi visual.

---

## 8. UI Assets

UI utama sebaiknya dibangun menggunakan Bevy UI, bukan dibuat sebagai satu gambar besar.

### UI yang direncanakan

- HUD.
- Company dashboard.
- Finance/report.
- Product development.
- Employee list.
- Hiring.
- Market.
- Inventory/building (future).

### Assets yang perlu disiapkan

| Asset | Format | Prioritas |
|---|---|---|
| UI icon | PNG | MVP |
| Company logo placeholder | PNG | Optional |
| Employee portrait | PNG | MVP untuk UI employee |
| Product placeholder image | PNG | Optional |
| UI font | TTF/OTF | MVP |
| Background/illustration | PNG | Optional |

### Icon specification

- PNG.
- Transparan.
- Konsisten dalam satu style.
- Idealnya SVG sebagai source desain, lalu PNG bila pipeline membutuhkan raster.
- Ukuran source 256×256 atau 512×512 sebagai titik awal, bukan ukuran wajib.
- Jangan menggunakan icon dengan style yang campur aduk.

### UI icon awal

- Money / finance.
- Company.
- Employee.
- Product.
- Time.
- Settings.
- Interaction indicator.

### UI layout principle

- Gunakan panel, text, button, chart, dan layout native.
- PNG hanya untuk icon, portrait, logo, atau ilustrasi.
- Jangan render seluruh UI menjadi gambar statis.
- Prioritaskan readability.

---

## 9. Audio

Audio tidak perlu menjadi blocker untuk M1.

### Future audio list

| Audio | Prioritas |
|---|---|
| UI click | Low |
| Door open/close | Low |
| Footsteps | Low |
| Keyboard typing | Low |
| Office ambience | Low |
| Background music | Low |
| Notification | Low |
| Object interaction | Low |

**Format runtime:** OGG (praktis untuk audio terkompresi).  
**Source:** WAV, jika tersedia.

---

## 10. Asset Sourcing

### Recommended starting sources

| Source | Penggunaan | URL |
|---|---|---|
| Kenney | Game assets, props, UI, konsistensi style | https://kenney.nl/assets |
| Mixamo | Humanoid animation dan workflow rigging | https://www.mixamo.com |
| Blender | Modeling, rigging, material, animation, export | https://www.blender.org |
| Poly Haven | Model, texture, HDRI | https://polyhaven.com |
| Sketchfab | Model 3D; periksa license per aset | https://sketchfab.com |

> Sumber ini merupakan titik awal. Pastikan license, file format, quality, rig, dan kesesuaian style sebelum mengunduh.

### Asset sourcing checklist

Sebelum mengunduh aset:

- [ ] Style cocok dengan proyek.
- [ ] File GLB tersedia atau dapat diekspor.
- [ ] License jelas.
- [ ] Aset dapat dimodifikasi sesuai kebutuhan.
- [ ] Polycount/complexity wajar.
- [ ] Scale masuk akal.
- [ ] Material dan texture berfungsi.
- [ ] Rig/animation compatibility (untuk karakter) diperiksa.
- [ ] Source file tersedia jika memungkinkan.
- [ ] Aset tidak memiliki bagian yang tidak diperlukan.

---

## 11. Asset Folder Structure

Struktur ini bersifat usulan. Buat folder sesuai aset yang benar-benar sudah digunakan, jangan membuat seluruh folder kosong untuk aset yang belum dibutuhkan.

```text
assets/
├── characters/
│   ├── player/
│   │   ├── founder.glb
│   │   └── founder.blend
│   ├── employees/
│   │   ├── developer.glb
│   │   └── portraits/
│   │       └── developer_01.png
│   └── animations/
├── environment/
│   ├── floor/
│   ├── walls/
│   ├── doors/
│   └── windows/
├── furniture/
│   ├── desk.glb
│   ├── chair.glb
│   └── cabinet.glb
├── equipment/
│   ├── computer.glb
│   └── monitor.glb
├── ui/
│   ├── icons/
│   ├── portraits/
│   └── logos/
├── textures/
├── audio/
│   ├── music/
│   ├── ambient/
│   └── sfx/
└── fonts/
```

### Asset naming convention

Gunakan nama lowercase dan konsisten:

- `character_founder.glb`
- `character_developer.glb`
- `furniture_desk_basic.glb`
- `furniture_chair_office.glb`
- `equipment_pc_basic.glb`
- `environment_wall_plain.glb`
- `ui_icon_finance.png`

Hindari nama seperti `final2_revisi_fix_baru.glb`.

---

## 12. Scale & Technical Preparation

Tetapkan ukuran standar sebelum membuat/mengumpulkan aset.

### Suggested scale conventions

- Gunakan satu unit standar yang konsisten untuk dunia game.
- Tentukan grid office (misalnya 1 meter per grid unit) sebelum implementasi building system.
- Periksa ukuran karakter, pintu, desk, chair, dan floor terhadap satu sama lain.
- Periksa orientation dan pivot.
- Periksa asset import di prototype Bevy.

> Angka grid 1 meter adalah usulan awal, bukan constraint final. Konsistensi lebih penting daripada angka spesifik.

### Blender export checklist

- [ ] Object transforms diperiksa.
- [ ] Scale/orientation konsisten.
- [ ] Origin/pivot ditentukan.
- [ ] Material diperiksa.
- [ ] Texture paths diperiksa.
- [ ] Rig/animation (jika ada) diperiksa.
- [ ] Export GLB.
- [ ] GLB dibuka/diuji ulang.
- [ ] Aset diuji di Bevy.

---

## 13. MVP Asset Checklist (Prioritized)

### Tier 0 — Required for first playable movement prototype (M1)

- [ ] Simple office floor.
- [ ] Wall modules atau temporary wall.
- [ ] Door.
- [ ] Simple lighting.
- [ ] First-person camera/player capsule (boleh tanpa model).
- [ ] Placeholder material.

### Tier 1 — Required for interaction & office prototype (M2)

- [ ] Desk.
- [ ] Chair.
- [ ] Computer/monitor.
- [ ] Basic interaction indicator UI.
- [ ] Door/interactable object jika masuk scope.

### Tier 2 — Required for company prototype

- [ ] Employee character (Developer).
- [ ] Idle animation.
- [ ] Sitting/working animation (jika tersedia).
- [ ] Employee portrait.
- [ ] UI icons.
- [ ] UI font.

### Tier 3 — Nice-to-have / polish

- [ ] Founder model.
- [ ] Keyboard/mouse.
- [ ] Cabinet.
- [ ] Plant.
- [ ] Trash bin.
- [ ] Office ambience.
- [ ] UI click sound.
- [ ] Extra props.

---

## 14. Recommended Asset Collection Strategy

Jangan mengumpulkan seluruh asset catalog di awal.

### Step 1 — Build one coherent scene

Kumpulkan hanya aset untuk satu ruang kecil:

- Floor.
- Walls.
- Door.
- Window.
- Desk.
- Chair.
- Computer.
- Lamp/light.

### Step 2 — Check visual consistency

Tempatkan semua aset di satu scene Blender atau Bevy prototype.

Evaluasi:

- Apakah style konsisten?
- Apakah scale konsisten?
- Apakah detail terlalu ramai?
- Apakah material cocok?
- Apakah aset cukup ringan?
- Apakah ruang terasa nyaman untuk eksplorasi first-person?

### Step 3 — Replace placeholders gradually

Mulai dengan primitive/placeholder. Ganti dengan aset final setelah fungsi dan scale sudah benar.

### Step 4 — Expand only when feature requires it

Kumpulkan aset berdasarkan milestone, bukan berdasarkan keinginan mengoleksi.

---

## 15. Milestone Integration

| Milestone | Asset / visual yang diperlukan |
|---|---|
| M0 — Preproduction | Visual reference, asset pipeline, folder conventions |
| M1 — World & Player | Floor, walls, lighting, door, camera, placeholders |
| M2 — Interaction | Desk, chair, PC, interactable objects |
| M3 — Company & Finance | UI icon, font, UI placeholder |
| M4 — Product | Product UI / placeholder |
| M5 — Employee | Employee 3D, working animation, portrait |
| M6 — Market | Dashboard UI, chart visuals |
| M7 — Management UI | Icons, panels, fonts |
| M8 — Office Expansion | Modular walls, furniture, buildable objects |
| M9 — Save System | Asset/layout identifiers and persistence validation |
| M10 — Polish | Audio, effects, visual polish, optimization |

---

## 16. Asset Registry

Catat aset yang digunakan agar mudah dirawat.

| Asset name | Category | Source | Format | License | Status | Notes |
|---|---|---|---|---|---|---|
| Office floor | Environment | TBD | GLB | TBD | Needed | Modular |
| Wall module | Environment | TBD | GLB | TBD | Needed | Grid-compatible |
| Door | Environment | TBD | GLB | TBD | Needed | Interactable later |
| Desk | Furniture | TBD | GLB | TBD | Needed | Basic |
| Office chair | Furniture | TBD | GLB | TBD | Needed | Basic |
| Computer | Equipment | TBD | GLB | TBD | Needed | Basic |
| Developer | Character | TBD | GLB | TBD | Future | Rigged |
| Employee portrait | UI | TBD | PNG | TBD | Future | Profile |
| UI icons | UI | TBD | PNG | TBD | Future | Consistent set |

Status yang disarankan: `Needed`, `Searching`, `Selected`, `Imported`, `Tested`, `Rejected`.

---

## 17. Definition of Ready — Assets

Aset siap diintegrasikan apabila:

- [ ] File dan format diketahui.
- [ ] License/source tercatat.
- [ ] Style cocok.
- [ ] Scale dan orientation diperiksa.
- [ ] File runtime tersedia.
- [ ] Texture tidak missing.
- [ ] Rig/animation diuji (untuk character).
- [ ] Tidak mengandung geometry/texture yang tidak perlu.
- [ ] Dapat di-load di prototype.
- [ ] Sudah diuji di Bevy bila menjadi asset runtime.

---

## 18. Definition of Done — Asset Integration

Integrasi asset dianggap selesai apabila:

- [ ] Asset di-load dan ditampilkan dengan benar.
- [ ] Scale/orientation/pivot benar.
- [ ] Material/texture berfungsi.
- [ ] Tidak ada error import yang menghambat.
- [ ] Performa scene masih wajar.
- [ ] Asset dapat digunakan oleh fitur yang dituju.
- [ ] Source/license dicatat.
- [ ] Git checkpoint dibuat.

---

## 19. Suggested Initial Shopping/Download List

Ini bukan daftar untuk mengunduh semuanya sekaligus. Mulai dari kebutuhan M1/M2.

### Immediate

1. Floor module atau floor placeholder.
2. Wall module atau wall placeholder.
3. Door.
4. Basic lighting setup.
5. Desk.
6. Office chair.
7. Computer/monitor.

### Later

1. Developer character.
2. Developer animation (idle, sitting/typing).
3. Employee portrait.
4. UI icon set.
5. Font.
6. Office props and audio.

---

## 20. Final Recommendation

Prioritas awal bukan jumlah aset, melainkan **satu scene kantor kecil yang konsisten, terukur, dan dapat berjalan di Bevy**.

Urutan kerja:

1. Tentukan visual style dan scale.
2. Buat scene kantor prototype sederhana.
3. Implementasi M1 menggunakan placeholder bila perlu.
4. Integrasikan aset modular untuk M2.
5. Tambahkan character saat employee system mulai dibangun.
6. Tambahkan UI assets ketika management UI masuk milestone.
7. Tambahkan audio dan polish di akhir.

Dengan pendekatan ini, pengembangan tidak terhambat oleh asset gathering dan aset yang sudah dikumpulkan memiliki kebutuhan implementasi yang jelas.
