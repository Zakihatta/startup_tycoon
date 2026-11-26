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
    make_uv_sphere,
    merge_meshes,
    rotate_y_mesh,
)


# ---------------------------------------------------------------------------
# Environment (Tier 0 - M1)
# ---------------------------------------------------------------------------

def build_floor_tile(out_path: str) -> None:
    b = GLBBuilder("FloorTileScene")
    mat_wood_a = b.add_material("FloorOakLight", (0.74, 0.57, 0.39, 1.0), roughness=0.68, metallic=0.0)
    mat_wood_b = b.add_material("FloorOakWarm", (0.66, 0.49, 0.33, 1.0), roughness=0.72, metallic=0.0)
    mat_grout = b.add_material("FloorSeam", (0.44, 0.31, 0.20, 1.0), roughness=0.88, metallic=0.0)

    # 1m x 1m hardwood plank tile, top surface at y = 0.0
    base = make_box(1.0, 0.05, 1.0, (0.0, -0.035, 0.0))
    planks_a = merge_meshes([
        make_box(0.985, 0.012, 0.23, (0.0, -0.006, -0.375)),
        make_box(0.985, 0.012, 0.23, (0.0, -0.006, 0.125)),
    ])
    planks_b = merge_meshes([
        make_box(0.985, 0.012, 0.23, (0.0, -0.006, -0.125)),
        make_box(0.985, 0.012, 0.23, (0.0, -0.006, 0.375)),
    ])
    mesh = b.add_mesh("FloorTileMesh", [(base, mat_grout), (planks_a, mat_wood_a), (planks_b, mat_wood_b)])
    b.add_node("FloorTile", mesh=mesh)
    b.build_glb(out_path)


def build_wall_plain(out_path: str) -> None:
    b = GLBBuilder("WallPlainScene")
    mat_wall = b.add_material("WallWarmPlaster", (0.91, 0.84, 0.75, 1.0), roughness=0.86, metallic=0.0)
    mat_baseboard = b.add_material("BaseboardOak", (0.46, 0.32, 0.20, 1.0), roughness=0.65, metallic=0.0)

    wall = make_box(1.0, 2.8, 0.15, (0.0, 1.4, 0.0))
    baseboard = make_box(1.0, 0.11, 0.168, (0.0, 0.055, 0.0))
    mesh = b.add_mesh("WallPlainMesh", [(wall, mat_wall), (baseboard, mat_baseboard)])
    b.add_node("WallPlain", mesh=mesh)
    b.build_glb(out_path)


def build_wall_window(out_path: str) -> None:
    b = GLBBuilder("WallWindowScene")
    mat_wall = b.add_material("WallWarmPlaster", (0.91, 0.84, 0.75, 1.0), roughness=0.86, metallic=0.0)
    mat_baseboard = b.add_material("BaseboardOak", (0.46, 0.32, 0.20, 1.0), roughness=0.65, metallic=0.0)
    mat_frame = b.add_material("WindowFrameWood", (0.36, 0.25, 0.17, 1.0), roughness=0.55, metallic=0.10)
    mat_glass = b.add_material(
        "WindowSunGlass",
        (0.98, 0.94, 0.86, 0.22),
        roughness=0.08,
        metallic=0.05,
        emissive=(0.14, 0.11, 0.07),
        alpha_mode="BLEND",
    )

    # Tall grid-mullioned studio window: x in [-0.40, 0.40], y in [0.58, 2.48]
    wall_geo = merge_meshes([
        make_box(1.0, 0.58, 0.15, (0.0, 0.29, 0.0)),        # lower sill wall
        make_box(1.0, 0.32, 0.15, (0.0, 2.64, 0.0)),        # top header wall
        make_box(0.10, 1.90, 0.15, (-0.45, 1.53, 0.0)),     # left pillar
        make_box(0.10, 1.90, 0.15, (0.45, 1.53, 0.0)),      # right pillar
    ])
    baseboard = make_box(1.0, 0.11, 0.168, (0.0, 0.055, 0.0))
    frame_geo = merge_meshes([
        make_box(0.84, 0.05, 0.175, (0.0, 0.58, 0.0)),      # bottom sill
        make_box(0.84, 0.05, 0.175, (0.0, 2.48, 0.0)),      # top arch header
        make_box(0.05, 1.90, 0.175, (-0.40, 1.53, 0.0)),    # left jamb
        make_box(0.05, 1.90, 0.175, (0.40, 1.53, 0.0)),     # right jamb
        # Grid muntin bars so sunlight casts signature grid shadow patterns
        make_box(0.028, 1.88, 0.06, (-0.135, 1.53, 0.0)),
        make_box(0.028, 1.88, 0.06, (0.135, 1.53, 0.0)),
        make_box(0.78, 0.028, 0.06, (0.0, 1.05, 0.0)),
        make_box(0.78, 0.028, 0.06, (0.0, 1.53, 0.0)),
        make_box(0.78, 0.028, 0.06, (0.0, 2.01, 0.0)),
    ])
    glass_geo = make_box(0.76, 1.86, 0.012, (0.0, 1.53, 0.0))

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
    mat_wall = b.add_material("WallWarmPlaster", (0.91, 0.84, 0.75, 1.0), roughness=0.86, metallic=0.0)
    mat_baseboard = b.add_material("BaseboardOak", (0.46, 0.32, 0.20, 1.0), roughness=0.65, metallic=0.0)
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
    mat_wood = b.add_material("DeskWalnut", (0.59, 0.42, 0.27, 1.0), roughness=0.62, metallic=0.0)
    mat_leg = b.add_material("DeskOakLeg", (0.46, 0.32, 0.20, 1.0), roughness=0.68, metallic=0.0)

    # 1.40m wide x 0.70m deep x 0.75m tall warm wooden studio desk
    top = make_box(1.40, 0.04, 0.70, (0.0, 0.73, 0.0))
    frame = merge_meshes([
        make_box(0.05, 0.71, 0.05, (-0.64, 0.355, -0.29)),
        make_box(0.05, 0.71, 0.05, (0.64, 0.355, -0.29)),
        make_box(0.05, 0.71, 0.05, (-0.64, 0.355, 0.29)),
        make_box(0.05, 0.71, 0.05, (0.64, 0.355, 0.29)),
        make_box(0.04, 0.04, 0.58, (-0.64, 0.69, 0.0)),
        make_box(0.04, 0.04, 0.58, (0.64, 0.69, 0.0)),
        make_box(1.28, 0.14, 0.025, (0.0, 0.63, -0.29)),
    ])
    mesh = b.add_mesh("DeskBasicMesh", [(top, mat_wood), (frame, mat_leg)])
    b.add_node("DeskBasic", mesh=mesh)
    b.build_glb(out_path)


def build_chair_office(out_path: str) -> None:
    b = GLBBuilder("ChairOfficeScene")
    mat_fabric = b.add_material("ChairPeach", (0.86, 0.59, 0.45, 1.0), roughness=0.80, metallic=0.0)
    mat_frame = b.add_material("ChairFrame", (0.20, 0.19, 0.20, 1.0), roughness=0.45, metallic=0.45)
    mat_accent = b.add_material("ChairAccent", (0.94, 0.76, 0.62, 1.0), roughness=0.60, metallic=0.0)

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
    mat_body = b.add_material("MonitorPlastic", (0.15, 0.15, 0.17, 1.0), roughness=0.45, metallic=0.20)
    mat_stand = b.add_material("MonitorAluminum", (0.70, 0.68, 0.66, 1.0), roughness=0.32, metallic=0.72)
    mat_screen = b.add_material(
        "MonitorScreenGlow",
        (0.28, 0.64, 0.88, 1.0),
        roughness=0.12,
        metallic=0.05,
        emissive=(0.18, 0.44, 0.72),
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


# ---------------------------------------------------------------------------
# Future / Expansion Furniture & Equipment (M8)
# ---------------------------------------------------------------------------

def build_sofa_basic(out_path: str) -> None:
    b = GLBBuilder("SofaBasicScene")
    mat_fabric = b.add_material("SofaWarmSand", (0.82, 0.72, 0.60, 1.0), roughness=0.84, metallic=0.0)
    mat_wood = b.add_material("SofaOakWood", (0.52, 0.36, 0.22, 1.0), roughness=0.65, metallic=0.0)
    mat_pillow_y = b.add_material("PillowYellow", (0.94, 0.80, 0.34, 1.0), roughness=0.85, metallic=0.0)
    mat_pillow_b = b.add_material("PillowTeal", (0.36, 0.56, 0.68, 1.0), roughness=0.85, metallic=0.0)

    # 1.50m wide x 0.74m deep x 0.76m tall, faces +Z
    legs = merge_meshes([
        make_cylinder(0.025, 0.018, 0.14, 8, (-0.66, 0.07, 0.28)),
        make_cylinder(0.025, 0.018, 0.14, 8, (0.66, 0.07, 0.28)),
        make_cylinder(0.025, 0.018, 0.14, 8, (-0.66, 0.07, -0.28)),
        make_cylinder(0.025, 0.018, 0.14, 8, (0.66, 0.07, -0.28)),
    ])
    cushions = merge_meshes([
        make_box(1.44, 0.14, 0.68, (0.0, 0.21, 0.0)),      # base frame
        make_box(0.62, 0.12, 0.60, (-0.32, 0.34, 0.03)),   # left seat cushion
        make_box(0.62, 0.12, 0.60, (0.32, 0.34, 0.03)),    # right seat cushion
        make_tapered_box(1.40, 0.16, 1.36, 0.13, 0.42, (0.0, 0.54, -0.24)),  # backrest
        make_box(0.14, 0.30, 0.68, (-0.68, 0.41, 0.0)),    # left armrest
        make_box(0.14, 0.30, 0.68, (0.68, 0.41, 0.0)),     # right armrest
    ])
    pillow_y = make_tapered_box(0.24, 0.10, 0.20, 0.08, 0.24, (-0.42, 0.48, -0.06))
    pillow_b = make_tapered_box(0.24, 0.10, 0.20, 0.08, 0.24, (0.42, 0.48, -0.06))
    mesh = b.add_mesh("SofaBasicMesh", [
        (legs, mat_wood),
        (cushions, mat_fabric),
        (pillow_y, mat_pillow_y),
        (pillow_b, mat_pillow_b),
    ])
    b.add_node("SofaBasic", mesh=mesh)
    b.build_glb(out_path)


def build_plant_pot(out_path: str) -> None:
    b = GLBBuilder("PlantPotScene")
    mat_pot = b.add_material("CeramicCream", (0.93, 0.90, 0.85, 1.0), roughness=0.45, metallic=0.0)
    mat_pattern = b.add_material("CeramicCobalt", (0.28, 0.46, 0.72, 1.0), roughness=0.45, metallic=0.0)
    mat_soil = b.add_material("PlantSoil", (0.24, 0.17, 0.12, 1.0), roughness=0.95, metallic=0.0)
    mat_stem = b.add_material("PlantStem", (0.38, 0.50, 0.26, 1.0), roughness=0.80, metallic=0.0)
    mat_leaf = b.add_material("PlantLeafWarm", (0.48, 0.68, 0.34, 1.0), roughness=0.70, metallic=0.0)

    pot = make_cylinder(0.15, 0.19, 0.36, 14, (0.0, 0.18, 0.0))
    # Decorative cobalt ceramic pattern spots around pot
    pot_spots = merge_meshes([
        make_uv_sphere(0.036, 0.038, 0.036, 6, 8, (0.175 * math.cos(i * math.pi / 3.0), 0.19, 0.175 * math.sin(i * math.pi / 3.0)))
        for i in range(6)
    ])
    soil = make_cylinder(0.17, 0.17, 0.04, 12, (0.0, 0.35, 0.0))
    stems = merge_meshes([
        make_cylinder(0.018, 0.014, 0.52, 8, (0.0, 0.58, 0.0)),
        make_cylinder(0.014, 0.010, 0.40, 8, (-0.07, 0.52, 0.05)),
        make_cylinder(0.014, 0.010, 0.42, 8, (0.07, 0.54, -0.05)),
    ])
    leaves = merge_meshes([
        make_uv_sphere(0.15, 0.11, 0.15, 7, 10, (0.0, 0.86, 0.0)),
        make_uv_sphere(0.13, 0.08, 0.13, 6, 9, (-0.13, 0.72, 0.08)),
        make_uv_sphere(0.13, 0.08, 0.13, 6, 9, (0.13, 0.74, -0.08)),
        make_uv_sphere(0.11, 0.07, 0.11, 6, 9, (0.07, 0.66, 0.13)),
        make_uv_sphere(0.11, 0.07, 0.11, 6, 9, (-0.08, 0.64, -0.11)),
    ])
    mesh = b.add_mesh("PlantPotMesh", [
        (pot, mat_pot),
        (pot_spots, mat_pattern),
        (soil, mat_soil),
        (stems, mat_stem),
        (leaves, mat_leaf),
    ])
    b.add_node("PlantPot", mesh=mesh)
    b.build_glb(out_path)


def build_whiteboard(out_path: str) -> None:
    b = GLBBuilder("WhiteboardScene")
    mat_frame = b.add_material("WhiteboardSteel", (0.20, 0.22, 0.26, 1.0), roughness=0.45, metallic=0.60)
    mat_board = b.add_material("WhiteboardSurface", (0.96, 0.97, 0.98, 1.0), roughness=0.25, metallic=0.0)
    mat_note1 = b.add_material("StickyYellow", (0.98, 0.84, 0.28, 1.0), roughness=0.75, metallic=0.0)
    mat_note2 = b.add_material("StickyTeal", (0.24, 0.78, 0.64, 1.0), roughness=0.75, metallic=0.0)

    frame = merge_meshes([
        make_box(0.04, 1.72, 0.04, (-0.58, 0.86, 0.0)),     # left post
        make_box(0.04, 1.72, 0.04, (0.58, 0.86, 0.0)),      # right post
        make_box(0.05, 0.04, 0.44, (-0.58, 0.02, 0.0)),     # left foot
        make_box(0.05, 0.04, 0.44, (0.58, 0.02, 0.0)),      # right foot
        make_box(1.16, 0.03, 0.08, (0.0, 0.84, 0.03)),      # marker tray
    ])
    board = make_box(1.12, 0.82, 0.03, (0.0, 1.28, 0.0))
    notes_y = merge_meshes([
        make_box(0.11, 0.11, 0.008, (-0.34, 1.44, 0.02)),
        make_box(0.11, 0.11, 0.008, (-0.19, 1.44, 0.02)),
    ])
    notes_t = merge_meshes([
        make_box(0.11, 0.11, 0.008, (-0.04, 1.44, 0.02)),
        make_box(0.28, 0.10, 0.008, (0.24, 1.26, 0.02)),
    ])
    mesh = b.add_mesh("WhiteboardMesh", [
        (frame, mat_frame),
        (board, mat_board),
        (notes_y, mat_note1),
        (notes_t, mat_note2),
    ])
    b.add_node("Whiteboard", mesh=mesh)
    b.build_glb(out_path)


def build_server_rack(out_path: str) -> None:
    b = GLBBuilder("ServerRackScene")
    mat_cab = b.add_material("RackCabinet", (0.12, 0.13, 0.16, 1.0), roughness=0.45, metallic=0.45)
    mat_blade = b.add_material("ServerBlade", (0.20, 0.22, 0.26, 1.0), roughness=0.50, metallic=0.35)
    mat_led_g = b.add_material(
        "ServerLedGreen",
        (0.18, 0.90, 0.56, 1.0),
        roughness=0.20,
        metallic=0.0,
        emissive=(0.14, 0.72, 0.45),
    )
    mat_led_b = b.add_material(
        "ServerLedBlue",
        (0.24, 0.64, 0.98, 1.0),
        roughness=0.20,
        metallic=0.0,
        emissive=(0.18, 0.48, 0.85),
    )

    cabinet = make_box(0.58, 1.48, 0.58, (0.0, 0.74, 0.0))
    blades = merge_meshes([
        make_box(0.50, 0.14, 0.02, (0.0, 0.26 + i * 0.20, 0.29))
        for i in range(6)
    ])
    leds_g = merge_meshes([
        make_box(0.12, 0.015, 0.012, (-0.14, 0.26 + i * 0.20, 0.305))
        for i in range(0, 6, 2)
    ])
    leds_b = merge_meshes([
        make_box(0.12, 0.015, 0.012, (-0.14, 0.26 + i * 0.20, 0.305))
        for i in range(1, 6, 2)
    ])
    mesh = b.add_mesh("ServerRackMesh", [
        (cabinet, mat_cab),
        (blades, mat_blade),
        (leds_g, mat_led_g),
        (leds_b, mat_led_b),
    ])
    b.add_node("ServerRack", mesh=mesh)
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
        ("assets/furniture/furniture_sofa_basic.glb", build_sofa_basic),
        ("assets/furniture/furniture_plant_pot.glb", build_plant_pot),
        ("assets/furniture/furniture_whiteboard.glb", build_whiteboard),
        ("assets/equipment/equipment_monitor_basic.glb", build_monitor_basic),
        ("assets/equipment/equipment_pc_basic.glb", build_pc_basic),
        ("assets/equipment/equipment_keyboard_mouse.glb", build_keyboard_mouse),
        ("assets/equipment/equipment_server_rack.glb", build_server_rack),
    ]
    for path, fn in targets:
        fn(path)
        print(f"Generated {path} ({os.path.getsize(path)} bytes)")


if __name__ == "__main__":
    main()
