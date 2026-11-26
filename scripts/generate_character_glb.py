"""
Procedural 3D Stylized Character (.glb) Generator
Redesigned for v0.2-B to match the warm stylized reference sheet:
- Friendly stylized head proportions, tousled dark hair, pill eyes + brows
- Rolled-up button-down sleeves, belt + metallic buckle, wristwatch on left wrist
- Rolled-cuff slim trousers and thick-soled sneakers
Outputs:
- assets/characters/player/character_founder.glb (Idle, Walk)
- assets/characters/employees/character_developer.glb (Idle, Walk, Sitting, Typing)
- assets/characters/courier/character_courier.glb (Idle, Walk)
"""

import math
import os
from dataclasses import dataclass
from typing import List, Tuple

from glb_lib import (
    GLBBuilder,
    MeshData,
    euler_to_quat,
    make_box,
    make_cylinder,
    make_tapered_box,
    make_uv_sphere,
    merge_meshes,
)


def rotate_mesh_xyz(mesh: MeshData, rx: float, ry: float, rz: float, offset: Tuple[float, float, float] = (0.0, 0.0, 0.0)) -> MeshData:
    """Rotate mesh vertices/normals around origin then translate by offset."""
    pos, norm, idx = mesh
    cx, sx = math.cos(rx), math.sin(rx)
    cy, sy = math.cos(ry), math.sin(ry)
    cz, sz = math.cos(rz), math.sin(rz)

    def xform(v: Tuple[float, float, float], translate: bool) -> Tuple[float, float, float]:
        x, y, z = v
        # X
        y1 = y * cx - z * sx
        z1 = y * sx + z * cx
        # Y
        x2 = x * cy + z1 * sy
        z2 = -x * sy + z1 * cy
        # Z
        x3 = x2 * cz - y1 * sz
        y3 = x2 * sz + y1 * cz
        if translate:
            return (x3 + offset[0], y3 + offset[1], z2 + offset[2])
        return (x3, y3, z2)

    return ([xform(p, True) for p in pos], [xform(n, False) for n in norm], idx)


@dataclass
class CharacterStyle:
    scene_name: str
    skin: Tuple[float, float, float, float]
    hair: Tuple[float, float, float, float]
    shirt: Tuple[float, float, float, float]
    cuff_shirt: Tuple[float, float, float, float]
    pants: Tuple[float, float, float, float]
    belt: Tuple[float, float, float, float]
    shoes: Tuple[float, float, float, float]
    sole: Tuple[float, float, float, float]
    has_watch: bool
    has_cap: bool
    cap_color: Tuple[float, float, float, float]
    has_parcel: bool
    include_work_anims: bool


def build_hair_tufts() -> MeshData:
    parts: List[MeshData] = [
        # Main rounded hair volume
        make_uv_sphere(0.148, 0.130, 0.152, rings=10, sectors=16, center=(0.0, 0.215, -0.015)),
        # Back & side coverage
        make_uv_sphere(0.140, 0.110, 0.125, rings=8, sectors=14, center=(0.0, 0.165, -0.045)),
        # Tousled front bangs sweeping down slightly
        rotate_mesh_xyz(make_tapered_box(0.075, 0.045, 0.020, 0.018, 0.085), -0.45, 0.0, 0.22, (-0.060, 0.235, 0.120)),
        rotate_mesh_xyz(make_tapered_box(0.082, 0.048, 0.022, 0.018, 0.095), -0.40, 0.0, -0.15, (0.015, 0.242, 0.126)),
        rotate_mesh_xyz(make_tapered_box(0.068, 0.042, 0.018, 0.016, 0.080), -0.45, 0.0, -0.35, (0.078, 0.230, 0.115)),
        # Top crown tousled tufts
        rotate_mesh_xyz(make_tapered_box(0.085, 0.065, 0.022, 0.020, 0.090), -0.25, 0.15, -0.35, (-0.045, 0.325, 0.020)),
        rotate_mesh_xyz(make_tapered_box(0.085, 0.065, 0.020, 0.020, 0.095), -0.15, -0.10, 0.28, (0.042, 0.330, 0.010)),
        rotate_mesh_xyz(make_tapered_box(0.075, 0.060, 0.020, 0.018, 0.080), 0.35, 0.0, 0.0, (0.0, 0.315, -0.075)),
    ]
    return merge_meshes(parts)


def build_character_glb(out_path: str, style: CharacterStyle) -> None:
    b = GLBBuilder(style.scene_name)

    mat_skin = b.add_material("Skin", style.skin, roughness=0.78, metallic=0.0)
    mat_hair = b.add_material("Hair", style.hair, roughness=0.86, metallic=0.0)
    mat_shirt = b.add_material("Shirt", style.shirt, roughness=0.82, metallic=0.0)
    mat_cuff = b.add_material("ShirtCuff", style.cuff_shirt, roughness=0.80, metallic=0.0)
    mat_pants = b.add_material("Pants", style.pants, roughness=0.84, metallic=0.0)
    mat_belt = b.add_material("Belt", style.belt, roughness=0.55, metallic=0.05)
    mat_buckle = b.add_material("Buckle", (0.82, 0.84, 0.88, 1.0), roughness=0.25, metallic=0.88)
    mat_shoes = b.add_material("Shoes", style.shoes, roughness=0.74, metallic=0.0)
    mat_sole = b.add_material("ShoeSole", style.sole, roughness=0.82, metallic=0.0)
    mat_eye = b.add_material("Eyes", (0.08, 0.07, 0.07, 1.0), roughness=0.30, metallic=0.05)
    mat_watch = b.add_material("WatchStrap", (0.20, 0.14, 0.11, 1.0), roughness=0.60, metallic=0.10)
    mat_cap = b.add_material("Cap", style.cap_color, roughness=0.78, metallic=0.0)
    mat_parcel = b.add_material("ParcelBox", (0.78, 0.60, 0.40, 1.0), roughness=0.88, metallic=0.0)
    mat_tape = b.add_material("ParcelTape", (0.90, 0.82, 0.64, 1.0), roughness=0.70, metallic=0.0)

    # --- Hips with Belt & Buckle ---
    hips_pants = make_tapered_box(0.25, 0.165, 0.255, 0.165, 0.135, (0.0, -0.015, 0.0))
    hips_belt = make_box(0.264, 0.038, 0.174, (0.0, 0.052, 0.0))
    hips_buckle = make_box(0.044, 0.042, 0.020, (0.0, 0.052, 0.086))
    mesh_hips = b.add_mesh("HipsMesh", [
        (hips_pants, mat_pants),
        (hips_belt, mat_belt),
        (hips_buckle, mat_buckle),
    ])

    # --- Torso: Button-down Shirt with Collar, Placket & Pocket ---
    torso_main = merge_meshes([
        make_tapered_box(0.260, 0.168, 0.315, 0.188, 0.37, (0.0, 0.20, 0.0)),
        make_uv_sphere(0.155, 0.055, 0.095, 6, 12, (0.0, 0.375, 0.0)),  # rounded shoulders
    ])
    torso_neck = make_cylinder(0.048, 0.045, 0.085, 10, (0.0, 0.415, 0.0))
    torso_details = merge_meshes([
        # Collar flaps
        rotate_mesh_xyz(make_box(0.075, 0.036, 0.068), 0.18, 0.25, -0.22, (-0.045, 0.388, 0.052)),
        rotate_mesh_xyz(make_box(0.075, 0.036, 0.068), 0.18, -0.25, 0.22, (0.045, 0.388, 0.052)),
        # Vertical front button placket
        make_box(0.024, 0.32, 0.012, (0.0, 0.21, 0.090)),
        # Left chest pocket
        make_box(0.060, 0.068, 0.014, (0.078, 0.245, 0.088)),
    ])
    torso_prims = [
        (torso_main, mat_shirt),
        (torso_details, mat_cuff),
        (torso_neck, mat_skin),
    ]
    if style.has_parcel:
        parcel_box = make_box(0.28, 0.20, 0.24, (0.0, 0.14, 0.22))
        parcel_tape = make_box(0.286, 0.035, 0.055, (0.0, 0.242, 0.22))
        torso_prims.append((parcel_box, mat_parcel))
        torso_prims.append((parcel_tape, mat_tape))
    mesh_torso = b.add_mesh("TorsoMesh", torso_prims)

    # --- Head: Larger Stylized Proportions, Pill Eyes, Brows & Tousled Hair ---
    head_skin = merge_meshes([
        make_uv_sphere(0.138, 0.150, 0.136, rings=12, sectors=16, center=(0.0, 0.155, 0.0)),
        make_uv_sphere(0.024, 0.032, 0.020, rings=6, sectors=8, center=(-0.136, 0.148, 0.0)),  # left ear
        make_uv_sphere(0.024, 0.032, 0.020, rings=6, sectors=8, center=(0.136, 0.148, 0.0)),   # right ear
        make_uv_sphere(0.014, 0.013, 0.015, rings=6, sectors=8, center=(0.0, 0.138, 0.136)),   # soft nose
    ])
    head_hair = build_hair_tufts()
    head_eyes = merge_meshes([
        # Pill-shaped expressive dark eyes
        make_uv_sphere(0.016, 0.024, 0.012, 8, 10, (-0.044, 0.160, 0.128)),
        make_uv_sphere(0.016, 0.024, 0.012, 8, 10, (0.044, 0.160, 0.128)),
        # Eyebrows
        rotate_mesh_xyz(make_box(0.038, 0.011, 0.010), 0.0, 0.0, 0.10, (-0.044, 0.200, 0.129)),
        rotate_mesh_xyz(make_box(0.038, 0.011, 0.010), 0.0, 0.0, -0.10, (0.044, 0.200, 0.129)),
    ])
    head_prims = [
        (head_skin, mat_skin),
        (head_hair, mat_hair),
        (head_eyes, mat_eye),
    ]
    if style.has_cap:
        cap_geo = merge_meshes([
            make_uv_sphere(0.150, 0.088, 0.150, 8, 14, (0.0, 0.245, -0.005)),
            rotate_mesh_xyz(make_tapered_box(0.19, 0.14, 0.16, 0.12, 0.020), 0.18, 0.0, 0.0, (0.0, 0.228, 0.145)),
        ])
        head_prims.append((cap_geo, mat_cap))
    mesh_head = b.add_mesh("HeadMesh", head_prims)

    # --- Arms: Rolled-up Shirt Sleeves on UpperArm, Bare Forearm + Wristwatch ---
    upper_sleeve = merge_meshes([
        make_uv_sphere(0.056, 0.056, 0.056, 8, 10, (0.0, 0.0, 0.0)),
        make_cylinder(0.046, 0.053, 0.21, 10, (0.0, -0.105, 0.0)),
    ])
    upper_cuff = make_cylinder(0.054, 0.054, 0.042, 10, (0.0, -0.215, 0.0))
    upper_skin_stub = make_cylinder(0.038, 0.040, 0.045, 10, (0.0, -0.245, 0.0))
    mesh_upper_arm = b.add_mesh("UpperArmMesh", [
        (upper_sleeve, mat_shirt),
        (upper_cuff, mat_cuff),
        (upper_skin_stub, mat_skin),
    ])

    forearm_skin = merge_meshes([
        make_uv_sphere(0.039, 0.039, 0.039, 8, 10, (0.0, 0.0, 0.0)),
        make_cylinder(0.033, 0.039, 0.21, 10, (0.0, -0.105, 0.0)),
        make_uv_sphere(0.034, 0.044, 0.028, 8, 10, (0.0, -0.235, 0.008)),  # hand
    ])
    mesh_forearm_plain = b.add_mesh("ForearmMesh", [(forearm_skin, mat_skin)])

    if style.has_watch:
        watch_band = make_cylinder(0.038, 0.038, 0.022, 10, (0.0, -0.185, 0.0))
        watch_face = make_cylinder(0.015, 0.015, 0.010, 10, (0.0, -0.185, 0.038))
        mesh_forearm_watch = b.add_mesh("ForearmWatchMesh", [
            (forearm_skin, mat_skin),
            (watch_band, mat_watch),
            (watch_face, mat_buckle),
        ])
    else:
        mesh_forearm_watch = mesh_forearm_plain

    # --- Legs: Slim Trousers with Rolled Cuffs + Brown/Cream Sneakers ---
    thigh_geo = merge_meshes([
        make_uv_sphere(0.068, 0.068, 0.068, 8, 10, (0.0, 0.0, 0.0)),
        make_cylinder(0.056, 0.068, 0.35, 10, (0.0, -0.175, 0.0)),
    ])
    mesh_thigh = b.add_mesh("ThighMesh", [(thigh_geo, mat_pants)])

    shin_pants = merge_meshes([
        make_uv_sphere(0.054, 0.054, 0.054, 8, 10, (0.0, 0.0, 0.0)),
        make_cylinder(0.046, 0.054, 0.30, 10, (0.0, -0.15, 0.0)),
        # Rolled pant cuff above ankle
        make_cylinder(0.052, 0.052, 0.045, 10, (0.0, -0.305, 0.0)),
    ])
    shin_ankle = make_cylinder(0.036, 0.038, 0.045, 10, (0.0, -0.34, 0.0))
    mesh_shin = b.add_mesh("ShinMesh", [
        (shin_pants, mat_pants),
        (shin_ankle, mat_skin),
    ])

    shoe_upper = merge_meshes([
        make_tapered_box(0.095, 0.21, 0.082, 0.16, 0.062, (0.0, -0.042, 0.045)),
        make_uv_sphere(0.046, 0.032, 0.052, 6, 10, (0.0, -0.048, 0.135)),
    ])
    shoe_sole = merge_meshes([
        make_box(0.104, 0.028, 0.235, (0.0, -0.082, 0.052)),
        make_uv_sphere(0.050, 0.015, 0.040, 6, 10, (0.0, -0.072, 0.145)),
    ])
    mesh_foot = b.add_mesh("FootMesh", [
        (shoe_upper, mat_shoes),
        (shoe_sole, mat_sole),
    ])

    # --- Node Hierarchy (identical joint names/offsets to preserve animation compatibility) ---
    n_foot_l = b.add_node("Foot_L", translation=(0.0, -0.36, 0.0), mesh=mesh_foot)
    n_shin_l = b.add_node("Shin_L", translation=(0.0, -0.36, 0.0), mesh=mesh_shin, children=[n_foot_l])
    n_thigh_l = b.add_node("Thigh_L", translation=(-0.088, -0.03, 0.0), mesh=mesh_thigh, children=[n_shin_l])

    n_foot_r = b.add_node("Foot_R", translation=(0.0, -0.36, 0.0), mesh=mesh_foot)
    n_shin_r = b.add_node("Shin_R", translation=(0.0, -0.36, 0.0), mesh=mesh_shin, children=[n_foot_r])
    n_thigh_r = b.add_node("Thigh_R", translation=(0.088, -0.03, 0.0), mesh=mesh_thigh, children=[n_shin_r])

    n_forearm_l = b.add_node("Forearm_L", translation=(0.0, -0.255, 0.0), mesh=mesh_forearm_plain)
    n_upper_arm_l = b.add_node("UpperArm_L", translation=(-0.205, 0.355, 0.0), mesh=mesh_upper_arm, children=[n_forearm_l])

    # In CharacterRoot rotated 180 deg Y, local +X is character's anatomical LEFT wrist -> wristwatch goes on Right node visually (character's left arm)!
    n_forearm_r = b.add_node("Forearm_R", translation=(0.0, -0.255, 0.0), mesh=mesh_forearm_watch)
    n_upper_arm_r = b.add_node("UpperArm_R", translation=(0.205, 0.355, 0.0), mesh=mesh_upper_arm, children=[n_forearm_r])

    n_head = b.add_node("Head", translation=(0.0, 0.44, 0.0), mesh=mesh_head)
    n_torso = b.add_node(
        "Torso",
        translation=(0.0, 0.05, 0.0),
        mesh=mesh_torso,
        children=[n_head, n_upper_arm_l, n_upper_arm_r],
    )

    HIP_BASE_Y = 0.845
    n_hips = b.add_node(
        "Hips",
        translation=(0.0, HIP_BASE_Y, 0.0),
        mesh=mesh_hips,
        children=[n_torso, n_thigh_l, n_thigh_r],
    )

    # Rotate CharacterRoot 180 deg around Y so local +Z faces Bevy -Z forward
    n_root = b.add_node(
        "CharacterRoot",
        translation=(0.0, 0.0, 0.0),
        rotation=euler_to_quat(0.0, math.pi, 0.0),
        children=[n_hips],
    )

    # --- Animation 0: Idle ---
    idle_dur = 2.4
    idle_steps = 24
    idle_times = [idle_dur * i / idle_steps for i in range(idle_steps + 1)]
    idle_hips_pos, idle_hips_rot, idle_torso_rot, idle_head_rot = [], [], [], []
    idle_arm_l_rot, idle_arm_r_rot, idle_forearm_l_rot, idle_forearm_r_rot = [], [], [], []

    for i in range(idle_steps + 1):
        p = 2.0 * math.pi * i / idle_steps
        breath = math.sin(p)
        sway = math.sin(p * 0.5)
        idle_hips_pos.append((0.0, HIP_BASE_Y + 0.007 * breath, 0.0))
        idle_hips_rot.append(euler_to_quat(0.0, 0.03 * sway, 0.015 * sway))
        idle_torso_rot.append(euler_to_quat(0.02 * breath, -0.02 * sway, -0.01 * sway))
        idle_head_rot.append(euler_to_quat(-0.02 * breath + 0.02 * math.sin(p), 0.05 * math.sin(p), 0.02 * sway))
        idle_arm_l_rot.append(euler_to_quat(0.03 * breath, 0.0, -0.08 - 0.012 * breath))
        idle_arm_r_rot.append(euler_to_quat(0.03 * breath, 0.0, 0.08 + 0.012 * breath))
        fore_bend = -0.55 if style.has_parcel else -0.18
        idle_forearm_l_rot.append(euler_to_quat(fore_bend - 0.03 * breath, 0.0, 0.0))
        idle_forearm_r_rot.append(euler_to_quat(fore_bend - 0.03 * breath, 0.0, 0.0))

    b.add_animation("Idle", [
        (n_hips, "translation", idle_times, idle_hips_pos),
        (n_hips, "rotation", idle_times, idle_hips_rot),
        (n_torso, "rotation", idle_times, idle_torso_rot),
        (n_head, "rotation", idle_times, idle_head_rot),
        (n_upper_arm_l, "rotation", idle_times, idle_arm_l_rot),
        (n_upper_arm_r, "rotation", idle_times, idle_arm_r_rot),
        (n_forearm_l, "rotation", idle_times, idle_forearm_l_rot),
        (n_forearm_r, "rotation", idle_times, idle_forearm_r_rot),
    ])

    # --- Animation 1: Walk ---
    walk_dur = 0.85
    walk_steps = 24
    walk_times = [walk_dur * i / walk_steps for i in range(walk_steps + 1)]
    w_hips_pos, w_hips_rot, w_torso_rot, w_head_rot = [], [], [], []
    w_thigh_l, w_shin_l, w_foot_l = [], [], []
    w_thigh_r, w_shin_r, w_foot_r = [], [], []
    w_uarm_l, w_farm_l, w_uarm_r, w_farm_r = [], [], [], []

    for i in range(walk_steps + 1):
        p = 2.0 * math.pi * i / walk_steps
        bob = -0.024 * abs(math.sin(p)) + 0.010
        sway_x = 0.016 * math.sin(p)
        w_hips_pos.append((sway_x, HIP_BASE_Y + bob, 0.0))
        w_hips_rot.append(euler_to_quat(0.03, 0.09 * math.sin(p), -0.035 * math.sin(p)))
        w_torso_rot.append(euler_to_quat(0.06, -0.12 * math.sin(p), 0.025 * math.sin(p)))
        w_head_rot.append(euler_to_quat(-0.04, 0.04 * math.sin(p), 0.0))

        for phase, thigh_arr, shin_arr, foot_arr in [
            (p, w_thigh_l, w_shin_l, w_foot_l),
            (p + math.pi, w_thigh_r, w_shin_r, w_foot_r),
        ]:
            thigh_rx = -0.52 * math.sin(phase)
            knee_rx = max(0.05, 0.68 * max(0.0, math.cos(phase)) + 0.16 * (1.0 - math.sin(phase)) * 0.3)
            foot_rx = -0.18 * math.sin(phase) - 0.14 * max(0.0, -math.cos(phase))
            thigh_arr.append(euler_to_quat(thigh_rx, 0.0, 0.0))
            shin_arr.append(euler_to_quat(knee_rx, 0.0, 0.0))
            foot_arr.append(euler_to_quat(foot_rx, 0.0, 0.0))

        if style.has_parcel:
            w_uarm_l.append(euler_to_quat(-0.35, 0.0, -0.08))
            w_uarm_r.append(euler_to_quat(-0.35, 0.0, 0.08))
            w_farm_l.append(euler_to_quat(-0.75, 0.0, 0.0))
            w_farm_r.append(euler_to_quat(-0.75, 0.0, 0.0))
        else:
            w_uarm_l.append(euler_to_quat(0.40 * math.sin(p), 0.0, -0.07))
            w_uarm_r.append(euler_to_quat(-0.40 * math.sin(p), 0.0, 0.07))
            w_farm_l.append(euler_to_quat(-0.42 - 0.20 * max(0.0, -math.sin(p)), 0.0, 0.0))
            w_farm_r.append(euler_to_quat(-0.42 - 0.20 * max(0.0, math.sin(p)), 0.0, 0.0))

    b.add_animation("Walk", [
        (n_hips, "translation", walk_times, w_hips_pos),
        (n_hips, "rotation", walk_times, w_hips_rot),
        (n_torso, "rotation", walk_times, w_torso_rot),
        (n_head, "rotation", walk_times, w_head_rot),
        (n_thigh_l, "rotation", walk_times, w_thigh_l),
        (n_shin_l, "rotation", walk_times, w_shin_l),
        (n_foot_l, "rotation", walk_times, w_foot_l),
        (n_thigh_r, "rotation", walk_times, w_thigh_r),
        (n_shin_r, "rotation", walk_times, w_shin_r),
        (n_foot_r, "rotation", walk_times, w_foot_r),
        (n_upper_arm_l, "rotation", walk_times, w_uarm_l),
        (n_forearm_l, "rotation", walk_times, w_farm_l),
        (n_upper_arm_r, "rotation", walk_times, w_uarm_r),
        (n_forearm_r, "rotation", walk_times, w_farm_r),
    ])

    if style.include_work_anims:
        HIP_SIT_Y = 0.515
        # --- Animation 2: Sitting ---
        sit_dur = 2.0
        sit_steps = 16
        sit_times = [sit_dur * i / sit_steps for i in range(sit_steps + 1)]
        s_hips_pos = [(0.0, HIP_SIT_Y, -0.02)] * (sit_steps + 1)
        s_hips_rot = [euler_to_quat(0.0, 0.0, 0.0)] * (sit_steps + 1)
        s_thigh = [euler_to_quat(-1.50, 0.0, 0.0)] * (sit_steps + 1)
        s_shin = [euler_to_quat(1.50, 0.0, 0.0)] * (sit_steps + 1)
        s_foot = [euler_to_quat(0.0, 0.0, 0.0)] * (sit_steps + 1)
        s_torso, s_head, s_uarm_l, s_uarm_r, s_farm_l, s_farm_r = [], [], [], [], [], []
        for i in range(sit_steps + 1):
            p = 2.0 * math.pi * i / sit_steps
            br = math.sin(p)
            s_torso.append(euler_to_quat(0.04 + 0.02 * br, 0.0, 0.0))
            s_head.append(euler_to_quat(0.04 - 0.02 * br, 0.06 * math.sin(p * 0.5), 0.0))
            s_uarm_l.append(euler_to_quat(-0.22, 0.0, -0.09))
            s_uarm_r.append(euler_to_quat(-0.22, 0.0, 0.09))
            s_farm_l.append(euler_to_quat(-0.85 + 0.03 * br, 0.0, 0.0))
            s_farm_r.append(euler_to_quat(-0.85 + 0.03 * br, 0.0, 0.0))

        b.add_animation("Sitting", [
            (n_hips, "translation", sit_times, s_hips_pos),
            (n_hips, "rotation", sit_times, s_hips_rot),
            (n_torso, "rotation", sit_times, s_torso),
            (n_head, "rotation", sit_times, s_head),
            (n_thigh_l, "rotation", sit_times, s_thigh),
            (n_shin_l, "rotation", sit_times, s_shin),
            (n_foot_l, "rotation", sit_times, s_foot),
            (n_thigh_r, "rotation", sit_times, s_thigh),
            (n_shin_r, "rotation", sit_times, s_shin),
            (n_foot_r, "rotation", sit_times, s_foot),
            (n_upper_arm_l, "rotation", sit_times, s_uarm_l),
            (n_upper_arm_r, "rotation", sit_times, s_uarm_r),
            (n_forearm_l, "rotation", sit_times, s_farm_l),
            (n_forearm_r, "rotation", sit_times, s_farm_r),
        ])

        # --- Animation 3: Typing ---
        typ_dur = 0.60
        typ_steps = 16
        typ_times = [typ_dur * i / typ_steps for i in range(typ_steps + 1)]
        t_hips_pos = [(0.0, HIP_SIT_Y, -0.02)] * (typ_steps + 1)
        t_hips_rot = [euler_to_quat(0.0, 0.0, 0.0)] * (typ_steps + 1)
        t_thigh = [euler_to_quat(-1.50, 0.0, 0.0)] * (typ_steps + 1)
        t_shin = [euler_to_quat(1.50, 0.0, 0.0)] * (typ_steps + 1)
        t_foot = [euler_to_quat(0.0, 0.0, 0.0)] * (typ_steps + 1)
        t_torso, t_head, t_uarm_l, t_uarm_r, t_farm_l, t_farm_r = [], [], [], [], [], []
        for i in range(typ_steps + 1):
            p = 2.0 * math.pi * i / typ_steps
            tap_l = math.sin(p * 2.0)
            tap_r = math.sin(p * 2.0 + math.pi)
            t_torso.append(euler_to_quat(0.10, 0.015 * math.sin(p), 0.0))
            t_head.append(euler_to_quat(0.13, 0.03 * math.sin(p), 0.0))
            t_uarm_l.append(euler_to_quat(-0.56 + 0.05 * tap_l, -0.10, -0.06))
            t_uarm_r.append(euler_to_quat(-0.56 + 0.05 * tap_r, 0.10, 0.06))
            t_farm_l.append(euler_to_quat(-0.90 + 0.16 * tap_l, 0.0, 0.0))
            t_farm_r.append(euler_to_quat(-0.90 + 0.16 * tap_r, 0.0, 0.0))

        b.add_animation("Typing", [
            (n_hips, "translation", typ_times, t_hips_pos),
            (n_hips, "rotation", typ_times, t_hips_rot),
            (n_torso, "rotation", typ_times, t_torso),
            (n_head, "rotation", typ_times, t_head),
            (n_thigh_l, "rotation", typ_times, t_thigh),
            (n_shin_l, "rotation", typ_times, t_shin),
            (n_foot_l, "rotation", typ_times, t_foot),
            (n_thigh_r, "rotation", typ_times, t_thigh),
            (n_shin_r, "rotation", typ_times, t_shin),
            (n_foot_r, "rotation", typ_times, t_foot),
            (n_upper_arm_l, "rotation", typ_times, t_uarm_l),
            (n_upper_arm_r, "rotation", typ_times, t_uarm_r),
            (n_forearm_l, "rotation", typ_times, t_farm_l),
            (n_forearm_r, "rotation", typ_times, t_farm_r),
        ])

    b.build_glb(out_path, root_nodes=[n_root])


def main() -> None:
    # Matches Reference Sheet 1: cream button-down, taupe trousers, dark tousled hair, brown sneakers
    founder_style = CharacterStyle(
        scene_name="FounderScene",
        skin=(0.94, 0.75, 0.62, 1.0),
        hair=(0.17, 0.12, 0.09, 1.0),
        shirt=(0.95, 0.93, 0.89, 1.0),
        cuff_shirt=(0.90, 0.87, 0.82, 1.0),
        pants=(0.45, 0.37, 0.30, 1.0),
        belt=(0.24, 0.17, 0.13, 1.0),
        shoes=(0.39, 0.29, 0.22, 1.0),
        sole=(0.88, 0.84, 0.78, 1.0),
        has_watch=True,
        has_cap=False,
        cap_color=(0.0, 0.0, 0.0, 1.0),
        has_parcel=False,
        include_work_anims=False,
    )

    # Matching cohesive warm studio look for employees
    dev_style = CharacterStyle(
        scene_name="DeveloperScene",
        skin=(0.91, 0.72, 0.59, 1.0),
        hair=(0.14, 0.10, 0.08, 1.0),
        shirt=(0.24, 0.34, 0.42, 1.0),
        cuff_shirt=(0.88, 0.85, 0.80, 1.0),
        pants=(0.28, 0.26, 0.25, 1.0),
        belt=(0.22, 0.16, 0.12, 1.0),
        shoes=(0.42, 0.31, 0.24, 1.0),
        sole=(0.88, 0.85, 0.80, 1.0),
        has_watch=False,
        has_cap=False,
        cap_color=(0.0, 0.0, 0.0, 1.0),
        has_parcel=False,
        include_work_anims=True,
    )

    # Delivery Courier in matching stylized proportions
    courier_style = CharacterStyle(
        scene_name="CourierScene",
        skin=(0.92, 0.73, 0.60, 1.0),
        hair=(0.20, 0.14, 0.10, 1.0),
        shirt=(0.20, 0.54, 0.42, 1.0),
        cuff_shirt=(0.90, 0.72, 0.28, 1.0),
        pants=(0.38, 0.33, 0.28, 1.0),
        belt=(0.22, 0.16, 0.12, 1.0),
        shoes=(0.36, 0.28, 0.22, 1.0),
        sole=(0.88, 0.85, 0.79, 1.0),
        has_watch=False,
        has_cap=True,
        cap_color=(0.94, 0.72, 0.24, 1.0),
        has_parcel=True,
        include_work_anims=False,
    )

    founder_out = os.path.join("assets", "characters", "player", "character_founder.glb")
    dev_out = os.path.join("assets", "characters", "employees", "character_developer.glb")
    courier_out = os.path.join("assets", "characters", "courier", "character_courier.glb")
    os.makedirs(os.path.dirname(courier_out), exist_ok=True)

    build_character_glb(founder_out, founder_style)
    build_character_glb(dev_out, dev_style)
    build_character_glb(courier_out, courier_style)
    print(f"Generated {founder_out} ({os.path.getsize(founder_out)} bytes)")
    print(f"Generated {dev_out} ({os.path.getsize(dev_out)} bytes)")
    print(f"Generated {courier_out} ({os.path.getsize(courier_out)} bytes)")


if __name__ == "__main__":
    main()
