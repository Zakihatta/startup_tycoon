"""
Procedural M1 & M2 Asset Pack (.glb) Generator for Startup Tycoon.
Follows docs/ASSETS_PREPARATION.md:
- Stylized modern 3D
- 1 meter = 1 unit scale
- Modular environment (floor, walls, window, door)
- Furniture & equipment (desk, office chair, monitor, PC, keyboard+mouse)
"""

import math
import os

from glb_lib import (
    GLBBuilder,
    make_box,
    make_cylinder,
    make_tapered_box,
    merge_meshes,
    rotate_y_mesh,
)


# ---------------------------------------------------------------------------
# Environment (Tier 0 - M1)
# ---------------------------------------------------------------------------

def build_floor_tile(out_path: str) -> None:
    b = GLBBuilder("FloorTileScene")
    mat_floor = b.add_material("FloorWarmBeige", (0.86, 0.82, 0.75, 1.0), roughness=0.78, metallic=0.0)
    mat_grout = b.add_material("FloorGroove", (0.74, 0.70, 0.63, 1.0), roughness=0.88, metallic=0.0)

    # 1m x 1m tile, top surface at y = 0.0
    base = make_box(1.0, 0.05, 1.0, (0.0, -0.035, 0.0))
    top_slab = make_box(0.985, 0.01, 0.985, (0.0, -0.005, 0.0))
    mesh = b.add_mesh("FloorTileMesh", [(base, mat_grout), (top_slab, mat_floor)])
    b.add_node("FloorTile", mesh=mesh)
    b.build_glb(out_path)


def build_wall_plain(out_path: str) -> None:
    b = GLBBuilder("WallPlainScene")
    mat_wall = b.add_material("WallWarmWhite", (0.94, 0.93, 0.90, 1.0), roughness=0.85, metallic=0.0)
    mat_baseboard = b.add_material("BaseboardGray", (0.35, 0.37, 0.40, 1.0), roughness=0.65, metallic=0.05)

    wall = make_box(1.0, 2.8, 0.15, (0.0, 1.4, 0.0))
    baseboard = make_box(1.0, 0.10, 0.165, (0.0, 0.05, 0.0))
    mesh = b.add_mesh("WallPlainMesh", [(wall, mat_wall), (baseboard, mat_baseboard)])
    b.add_node("WallPlain", mesh=mesh)
    b.build_glb(out_path)


def build_wall_window(out_path: str) -> None:
    b = GLBBuilder("WallWindowScene")
    mat_wall = b.add_material("WallWarmWhite", (0.94, 0.93, 0.90, 1.0), roughness=0.85, metallic=0.0)
    mat_baseboard = b.add_material("BaseboardGray", (0.35, 0.37, 0.40, 1.0), roughness=0.65, metallic=0.05)
    mat_frame = b.add_material("WindowFrame", (0.18, 0.20, 0.23, 1.0), roughness=0.45, metallic=0.35)
    mat_glass = b.add_material(
        "WindowGlass", (0.72, 0.86, 0.96, 0.35), roughness=0.10, metallic=0.05, alpha_mode="BLEND"
    )

    # Opening: x in [-0.36, 0.36] (0.72m wide), y in [0.95, 2.25] (1.30m tall)
    wall_geo = merge_meshes([
        make_box(1.0, 0.95, 0.15, (0.0, 0.475, 0.0)),       # bottom sill wall
        make_box(1.0, 0.55, 0.15, (0.0, 2.525, 0.0)),       # top header wall
        make_box(0.14, 1.30, 0.15, (-0.43, 1.60, 0.0)),     # left pillar
        make_box(0.14, 1.30, 0.15, (0.43, 1.60, 0.0)),      # right pillar
    ])
    baseboard = make_box(1.0, 0.10, 0.165, (0.0, 0.05, 0.0))
    frame_geo = merge_meshes([
        make_box(0.76, 0.04, 0.17, (0.0, 0.95, 0.0)),       # sill
        make_box(0.76, 0.04, 0.17, (0.0, 2.25, 0.0)),       # top frame
        make_box(0.04, 1.30, 0.17, (-0.36, 1.60, 0.0)),     # left frame
        make_box(0.04, 1.30, 0.17, (0.36, 1.60, 0.0)),      # right frame
        make_box(0.025, 1.26, 0.04, (0.0, 1.60, 0.0)),      # center mullion
    ])
    glass_geo = make_box(0.68, 1.26, 0.015, (0.0, 1.60, 0.0))

    mesh = b.add_mesh("WallWindowMesh", [
        (wall_geo, mat_wall),
        (baseboard, mat_baseboard),
        (frame_geo, mat_frame),
        (glass_geo, mat_glass),
    ])
    b.add_node("WallWindow", mesh=mesh)
    b.build_glb(out_path)


def build_wall_door(out_path: str) -> None:
    b = GLBBuilder("WallDoorScene")
    mat_wall = b.add_material("WallWarmWhite", (0.94, 0.93, 0.90, 1.0), roughness=0.85, metallic=0.0)
    mat_baseboard = b.add_material("BaseboardGray", (0.35, 0.37, 0.40, 1.0), roughness=0.65, metallic=0.05)
    mat_frame = b.add_material("DoorFrameWood", (0.45, 0.31, 0.20, 1.0), roughness=0.65, metallic=0.0)

    # Opening: x in [-0.42, 0.42] (0.84m wide), y in [0.0, 2.12]
    wall_geo = merge_meshes([
        make_box(1.0, 0.68, 0.15, (0.0, 2.46, 0.0)),        # header above door
        make_box(0.08, 2.12, 0.15, (-0.46, 1.06, 0.0)),     # left jamb wall
        make_box(0.08, 2.12, 0.15, (0.46, 1.06, 0.0)),      # right jamb wall
    ])
    baseboard = merge_meshes([
        make_box(0.08, 0.10, 0.165, (-0.46, 0.05, 0.0)),
        make_box(0.08, 0.10, 0.165, (0.46, 0.05, 0.0)),
    ])
    frame_geo = merge_meshes([
        make_box(0.88, 0.04, 0.175, (0.0, 2.12, 0.0)),      # top casing
        make_box(0.04, 2.12, 0.175, (-0.42, 1.06, 0.0)),    # left casing
        make_box(0.04, 2.12, 0.175, (0.42, 1.06, 0.0)),     # right casing
    ])
    mesh = b.add_mesh("WallDoorMesh", [
        (wall_geo, mat_wall),
        (baseboard, mat_baseboard),
        (frame_geo, mat_frame),
    ])
    b.add_node("WallDoor", mesh=mesh)
    b.build_glb(out_path)


def build_door_basic(out_path: str) -> None:
    b = GLBBuilder("DoorBasicScene")
    mat_door = b.add_material("DoorOak", (0.64, 0.46, 0.30, 1.0), roughness=0.60, metallic=0.0)
    mat_panel = b.add_material("DoorTrim", (0.56, 0.39, 0.25, 1.0), roughness=0.65, metallic=0.0)
    mat_handle = b.add_material("DoorHandle", (0.78, 0.80, 0.84, 1.0), roughness=0.25, metallic=0.85)

    # Pivot at hinge edge (x = 0.0, y = 0.0, z = 0.0)
    leaf = make_box(0.80, 2.09, 0.045, (0.40, 1.045, 0.0))
    trim = merge_meshes([
        make_box(0.58, 0.72, 0.055, (0.40, 1.50, 0.0)),
        make_box(0.58, 0.72, 0.055, (0.40, 0.56, 0.0)),
    ])
    handle = merge_meshes([
        make_box(0.11, 0.024, 0.11, (0.69, 1.02, 0.0)),
        make_cylinder(0.022, 0.022, 0.07, 10, (0.72, 1.02, 0.0)),
    ])
    mesh = b.add_mesh("DoorBasicMesh", [
        (leaf, mat_door),
        (trim, mat_panel),
        (handle, mat_handle),
    ])
    b.add_node("DoorHingeRoot", mesh=mesh)
    b.build_glb(out_path)


# ---------------------------------------------------------------------------
# Furniture (Tier 1 - M2)
# ---------------------------------------------------------------------------

def build_desk_basic(out_path: str) -> None:
    b = GLBBuilder("DeskBasicScene")
    mat_wood = b.add_material("DeskOak", (0.76, 0.58, 0.38, 1.0), roughness=0.58, metallic=0.0)
    mat_metal = b.add_material("DeskSteel", (0.16, 0.18, 0.21, 1.0), roughness=0.45, metallic=0.70)

    # 1.40m wide x 0.70m deep x 0.75m tall; origin at floor center
    top = make_box(1.40, 0.035, 0.70, (0.0, 0.7325, 0.0))
    frame = merge_meshes([
        # 4 legs
        make_box(0.045, 0.715, 0.045, (-0.64, 0.3575, -0.29)),
        make_box(0.045, 0.715, 0.045, (0.64, 0.3575, -0.29)),
        make_box(0.045, 0.715, 0.045, (-0.64, 0.3575, 0.29)),
        make_box(0.045, 0.715, 0.045, (0.64, 0.3575, 0.29)),
        # Side & back crossbars
        make_box(0.035, 0.035, 0.58, (-0.64, 0.69, 0.0)),
        make_box(0.035, 0.035, 0.58, (0.64, 0.69, 0.0)),
        make_box(1.28, 0.08, 0.025, (0.0, 0.62, -0.28)),
    ])
    mesh = b.add_mesh("DeskBasicMesh", [(top, mat_wood), (frame, mat_metal)])
    b.add_node("DeskBasic", mesh=mesh)
    b.build_glb(out_path)


def build_chair_office(out_path: str) -> None:
    b = GLBBuilder("ChairOfficeScene")
    mat_fabric = b.add_material("ChairFabric", (0.18, 0.22, 0.28, 1.0), roughness=0.82, metallic=0.0)
    mat_frame = b.add_material("ChairFrame", (0.14, 0.15, 0.18, 1.0), roughness=0.45, metallic=0.55)
    mat_accent = b.add_material("ChairAccent", (0.16, 0.65, 0.50, 1.0), roughness=0.50, metallic=0.10)

    # 5-star base + gas lift
    base_parts = [make_cylinder(0.028, 0.028, 0.36, 10, (0.0, 0.24, 0.0))]
    for i in range(5):
        ang = 2.0 * math.pi * i / 5.0
        leg = rotate_y_mesh(make_box(0.035, 0.03, 0.27, (0.0, 0.065, 0.135)), ang)
        wheel = rotate_y_mesh(make_cylinder(0.025, 0.025, 0.03, 8, (0.0, 0.025, 0.25)), ang)
        base_parts.extend([leg, wheel])

    # Armrests & spine
    frame_geo = merge_meshes(base_parts + [
        make_box(0.05, 0.36, 0.035, (0.0, 0.60, 0.23)),     # back spine (chair faces -Z)
        make_box(0.03, 0.18, 0.03, (-0.25, 0.54, 0.0)),     # left arm post
        make_box(0.03, 0.18, 0.03, (0.25, 0.54, 0.0)),      # right arm post
        make_box(0.05, 0.025, 0.24, (-0.25, 0.63, -0.02)),  # left arm pad
        make_box(0.05, 0.025, 0.24, (0.25, 0.63, -0.02)),   # right arm pad
    ])
    cushion_geo = merge_meshes([
        make_tapered_box(0.46, 0.44, 0.44, 0.42, 0.07, (0.0, 0.45, 0.0)),     # seat
        make_tapered_box(0.42, 0.06, 0.38, 0.05, 0.46, (0.0, 0.74, 0.20)),    # backrest
    ])
    accent_geo = make_box(0.32, 0.03, 0.02, (0.0, 0.94, 0.195))

    mesh = b.add_mesh("ChairOfficeMesh", [
        (frame_geo, mat_frame),
        (cushion_geo, mat_fabric),
        (accent_geo, mat_accent),
    ])
    b.add_node("ChairOffice", mesh=mesh)
    b.build_glb(out_path)


# ---------------------------------------------------------------------------
# Equipment (Tier 1 - M2)
# ---------------------------------------------------------------------------

def build_monitor_basic(out_path: str) -> None:
    b = GLBBuilder("MonitorBasicScene")
    mat_body = b.add_material("MonitorPlastic", (0.12, 0.13, 0.15, 1.0), roughness=0.45, metallic=0.25)
    mat_stand = b.add_material("MonitorAluminum", (0.68, 0.70, 0.74, 1.0), roughness=0.30, metallic=0.80)
    mat_screen = b.add_material(
        "MonitorScreen",
        (0.08, 0.14, 0.22, 1.0),
        roughness=0.15,
        metallic=0.10,
        emissive=(0.04, 0.09, 0.16),
    )

    # Origin at bottom of stand (y = 0.0), screen faces +Z
    stand = merge_meshes([
        make_box(0.22, 0.015, 0.18, (0.0, 0.0075, 0.0)),
        make_tapered_box(0.05, 0.04, 0.04, 0.03, 0.22, (0.0, 0.115, -0.05)),
    ])
    bezel = make_box(0.62, 0.36, 0.025, (0.0, 0.30, -0.02))
    screen = make_box(0.59, 0.33, 0.008, (0.0, 0.30, -0.006))

    mesh = b.add_mesh("MonitorBasicMesh", [
        (stand, mat_stand),
        (bezel, mat_body),
        (screen, mat_screen),
    ])
    b.add_node("MonitorBasic", mesh=mesh)
    b.build_glb(out_path)


def build_pc_basic(out_path: str) -> None:
    b = GLBBuilder("PCBasicScene")
    mat_case = b.add_material("PCCase", (0.15, 0.16, 0.19, 1.0), roughness=0.50, metallic=0.35)
    mat_front = b.add_material("PCFrontMesh", (0.09, 0.10, 0.12, 1.0), roughness=0.70, metallic=0.15)
    mat_led = b.add_material(
        "PCStatusLED",
        (0.15, 0.85, 0.55, 1.0),
        roughness=0.20,
        metallic=0.0,
        emissive=(0.12, 0.70, 0.45),
    )

    # Mid-tower PC: 0.21m x 0.42m x 0.40m, origin at bottom-center (y = 0.0)
    chassis = make_box(0.21, 0.42, 0.40, (0.0, 0.21, 0.0))
    front_panel = make_box(0.17, 0.36, 0.015, (0.0, 0.21, 0.20))
    led_strip = make_box(0.14, 0.012, 0.018, (0.0, 0.38, 0.202))

    mesh = b.add_mesh("PCBasicMesh", [
        (chassis, mat_case),
        (front_panel, mat_front),
        (led_strip, mat_led),
    ])
    b.add_node("PCBasic", mesh=mesh)
    b.build_glb(out_path)


def build_keyboard_mouse(out_path: str) -> None:
    b = GLBBuilder("KeyboardMouseScene")
    mat_body = b.add_material("PeripheralBody", (0.14, 0.15, 0.18, 1.0), roughness=0.55, metallic=0.10)
    mat_keys = b.add_material("Keycaps", (0.24, 0.26, 0.30, 1.0), roughness=0.70, metallic=0.0)

    kb_base = make_tapered_box(0.38, 0.14, 0.38, 0.14, 0.016, (-0.06, 0.008, 0.0))
    kb_keys = make_box(0.35, 0.012, 0.11, (-0.06, 0.018, 0.0))
    mouse = make_tapered_box(0.06, 0.105, 0.05, 0.09, 0.028, (0.21, 0.014, 0.01))

    mesh = b.add_mesh("KeyboardMouseMesh", [
        (merge_meshes([kb_base, mouse]), mat_body),
        (kb_keys, mat_keys),
    ])
    b.add_node("KeyboardMouse", mesh=mesh)
    b.build_glb(out_path)


def main() -> None:
    targets = [
        ("assets/environment/floor/environment_floor_tile.glb", build_floor_tile),
        ("assets/environment/walls/environment_wall_plain.glb", build_wall_plain),
        ("assets/environment/walls/environment_wall_window.glb", build_wall_window),
        ("assets/environment/walls/environment_wall_door.glb", build_wall_door),
        ("assets/environment/doors/environment_door_basic.glb", build_door_basic),
        ("assets/furniture/furniture_desk_basic.glb", build_desk_basic),
        ("assets/furniture/furniture_chair_office.glb", build_chair_office),
        ("assets/equipment/equipment_monitor_basic.glb", build_monitor_basic),
        ("assets/equipment/equipment_pc_basic.glb", build_pc_basic),
        ("assets/equipment/equipment_keyboard_mouse.glb", build_keyboard_mouse),
    ]
    for path, fn in targets:
        fn(path)
        print(f"Generated {path} ({os.path.getsize(path)} bytes)")


if __name__ == "__main__":
    main()
