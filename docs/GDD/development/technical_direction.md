# Technical Direction

## Proposed stack
Rust, Bevy, Cargo, Git, Serde, RON (tentative), Blender, VS Code atau IDE setara.

## Architecture principles
1. Pisahkan simulation/business logic dari rendering.
2. Systems kecil dan modular.
3. Gunakan data-oriented ECS sesuai kebutuhan.
4. Systems dapat diuji.
5. Stable identifiers untuk persistent objects.
6. Hindari monolithic modules.
7. Hindari dependencies yang tidak perlu.
8. Simulasi independen dari frame rate.

## Proposed module layout
```text
src/
├── main.rs
├── lib.rs
├── app/{mod.rs,plugins.rs,states.rs}
├── core/{mod.rs,time.rs,events.rs}
├── player/{mod.rs,movement.rs,camera.rs}
├── interaction/{mod.rs,system.rs}
├── company/{mod.rs,data.rs,progression.rs}
├── economy/{mod.rs,finance.rs,reports.rs}
├── product/{mod.rs,development.rs,data.rs}
├── employee/{mod.rs,data.rs,behavior.rs,hiring.rs}
├── market/{mod.rs,demand.rs,sales.rs}
├── office/{mod.rs,objects.rs,layout.rs}
├── ui/{mod.rs,hud.rs,company.rs,finance.rs,product.rs}
└── save_system/{mod.rs,persistence.rs}
```

## Technical caveats
Usulan layout, bukan struktur wajib. Adaptasikan dengan versi Bevy dan kompleksitas. Konfirmasi API/dependency compatibility sebelum implementasi. Jangan tambah plugin atau ubah version tanpa persetujuan.
