# Time System

## Purpose
Mengatur clock dan update simulasi yang membutuhkan waktu.

## Proposed features
Day counter, clock/time of day (optional), simulation tick, pause, speed controls.

## MVP
Time progression yang stabil, pause dan speed-up. Simulasi ekonomi/progress tidak boleh bergantung pada frame rate.

## Systems affected
Product progress, employee work/energy, salary, rent, market/sales, event triggers, reports.

## Design notes
Tentukan cadence setiap system (tick/hour/day). Tidak semua systems perlu update setiap frame. Simulasi sebaiknya reproducible/testable bila memungkinkan.
