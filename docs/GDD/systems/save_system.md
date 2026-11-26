# Save System

## Purpose
Menyimpan dan memulihkan state game secara konsisten.

## Persistent data
Player/game state, company, finance, employees, products, market (jika persistent), office layout, time, progression, settings (scope TBD).

## MVP
- Save and load.
- Basic validation/error handling.
- Persist core prototype data.
- Tentukan file location/format saat technical implementation.

## Proposed tooling
Serde dengan format serialization yang dipilih (RON atau lainnya; keputusan pending).

## Future
Auto-save, multiple slots, metadata, schema/version migration, backup/recovery.

## Constraints
Save data merepresentasikan simulation state, bukan transient render entities atau raw ECS entity IDs. Gunakan stable identifiers.
