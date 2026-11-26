# Startup Tycoon — Game Design Documents

Dokumentasi desain **Startup Tycoon**, game simulasi membangun dan mengelola perusahaan software/startup dalam lingkungan 3D first-person menggunakan Rust + Bevy.

## Dokumen
- [Main Concept](main_concept.md) — pitch, fantasy, pillars, scope.
- [Gameplay](gameplay.md) — loop, progression, flow.
- [Core Gameplay Systems](core_gameplay_systems.md) — overview sistem dan dependensi.
- [Company](systems/company_system.md)
- [Product](systems/product_system.md)
- [Employee](systems/employee_system.md)
- [Finance](systems/finance_system.md)
- [Market](systems/market_system.md)
- [Office](systems/office_system.md)
- [Events](systems/event_system.md)
- [Time](systems/time_system.md)
- [Interaction](systems/interaction_system.md)
- [UI](systems/ui_system.md)
- [Save](systems/save_system.md)
- [Roadmap](development/roadmap.md)
- [Technical Direction](development/technical_direction.md)
- [AI Vibe Coding Rules](development/ai_vibe_coding_rules.md)
- [MVP Scope](development/mvp_scope.md)
- [Definition of Done](development/definition_of_done.md)

## Status
Baseline GDD v1.0. Detail yang belum diputuskan ditandai *tentative*, *proposed*, atau *TBD*. Tidak semua sistem yang tercantum merupakan fitur MVP.

## Prinsip
- Build a small playable prototype first.
- Separate simulation logic from visuals.
- Implement milestone-by-milestone.
- Do not expand scope without agreement.
- Each milestone should be testable and reviewable.
