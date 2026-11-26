# Interaction System

## Purpose
Interaksi first-person dengan objek dan ruang kantor.

## MVP
- First-person movement.
- Camera.
- Interaction detection.
- Interaction prompt.
- Interact action.
- Workstation/PC interaction.

## Flow
1. Player berada dekat/menghadap objek.
2. Sistem menentukan target.
3. UI menampilkan prompt.
4. Player menekan input interact.
5. Action berjalan atau UI terbuka.
6. Prompt/state diperbarui.

## Future
Contextual interaction, door animation, furniture placement, multiple options, NPC interactions.

## Technical note
Gunakan interface/trait atau pola data modular. Hindari hard-coded object logic dalam satu system besar.
