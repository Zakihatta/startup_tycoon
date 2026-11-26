"""
Procedural 3D Character (.glb) Generator
Generates a stylized low-poly Startup Founder character with PBR materials
and two glTF 2.0 animations: 'Idle' and 'Walk'.
"""

import math
import os

from glb_lib import (
    GLBBuilder,
    euler_to_quat,
    make_box,
    make_cylinder,
    make_tapered_box,
    make_uv_sphere,
    merge_meshes,
)


def build_character_glb(out_path: str) -> None:
    b = GLBBuilder("CharacterScene")

    # Materials (Startup Founder look: navy blazer, crisp white tee, slate chinos, white sneakers)
    mat_skin = b.add_material("Skin", (0.92, 0.72, 0.58, 1.0), roughness=0.65, metallic=0.0)
    mat_hair = b.add_material("Hair", (0.12, 0.09, 0.08, 1.0), roughness=0.80, metallic=0.0)
    mat_blazer = b.add_material("Blazer", (0.13, 0.22, 0.38, 1.0), roughness=0.72, metallic=0.05)
    mat_shirt = b.add_material("Shirt", (0.94, 0.95, 0.97, 1.0), roughness=0.75, metallic=0.0)
    mat_pants = b.add_material("Pants", (0.20, 0.23, 0.28, 1.0), roughness=0.78, metallic=0.0)
    mat_shoes = b.add_material("Shoes", (0.93, 0.93, 0.95, 1.0), roughness=0.55, metallic=0.05)
    mat_sole = b.add_material("ShoeSole", (0.16, 0.18, 0.22, 1.0), roughness=0.85, metallic=0.0)
    mat_eye = b.add_material("Eyes", (0.06, 0.07, 0.09, 1.0), roughness=0.25, metallic=0.1)
    mat_glasses = b.add_material("Glasses", (0.10, 0.11, 0.14, 1.0), roughness=0.30, metallic=0.65)
    mat_badge = b.add_material("BadgeAccent", (0.16, 0.70, 0.52, 1.0), roughness=0.40, metallic=0.15)

    # --- Meshes ---
    mesh_hips = b.add_mesh("HipsMesh", [
        (make_tapered_box(0.28, 0.18, 0.27, 0.17, 0.13, (0.0, 0.0, 0.0)), mat_pants),
    ])

    torso_blazer = merge_meshes([
        make_tapered_box(0.29, 0.18, 0.36, 0.21, 0.40, (0.0, 0.22, 0.0)),
        make_box(0.05, 0.18, 0.03, (-0.065, 0.32, 0.105)),
        make_box(0.05, 0.18, 0.03, (0.065, 0.32, 0.105)),
    ])
    torso_shirt = merge_meshes([
        make_tapered_box(0.11, 0.03, 0.14, 0.03, 0.36, (0.0, 0.23, 0.095)),
        make_cylinder(0.055, 0.05, 0.08, 10, (0.0, 0.44, 0.0)),
    ])
    torso_badge = make_box(0.055, 0.075, 0.015, (0.09, 0.26, 0.112))
    mesh_torso = b.add_mesh("TorsoMesh", [
        (torso_blazer, mat_blazer),
        (torso_shirt, mat_shirt),
        (torso_badge, mat_badge),
    ])

    head_skin = merge_meshes([
        make_uv_sphere(0.125, 0.14, 0.13, rings=10, sectors=14, center=(0.0, 0.14, 0.0)),
        make_box(0.03, 0.05, 0.03, (-0.125, 0.135, 0.0)),
        make_box(0.03, 0.05, 0.03, (0.125, 0.135, 0.0)),
        make_box(0.025, 0.03, 0.025, (0.0, 0.125, 0.13)),
    ])
    head_hair = merge_meshes([
        make_uv_sphere(0.132, 0.11, 0.138, rings=8, sectors=14, center=(0.0, 0.20, -0.01)),
        make_box(0.24, 0.07, 0.08, (0.0, 0.26, 0.04)),
        make_box(0.25, 0.11, 0.10, (0.0, 0.16, -0.07)),
    ])
    head_eyes = merge_meshes([
        make_uv_sphere(0.016, 0.020, 0.012, 6, 8, (-0.045, 0.145, 0.122)),
        make_uv_sphere(0.016, 0.020, 0.012, 6, 8, (0.045, 0.145, 0.122)),
    ])
    head_glasses = merge_meshes([
        make_box(0.062, 0.042, 0.012, (-0.046, 0.145, 0.132)),
        make_box(0.062, 0.042, 0.012, (0.046, 0.145, 0.132)),
        make_box(0.030, 0.010, 0.012, (0.0, 0.152, 0.132)),
    ])
    mesh_head = b.add_mesh("HeadMesh", [
        (head_skin, mat_skin),
        (head_hair, mat_hair),
        (head_eyes, mat_eye),
        (head_glasses, mat_glasses),
    ])

    upper_arm_geo = merge_meshes([
        make_uv_sphere(0.055, 0.055, 0.055, 8, 10, (0.0, 0.0, 0.0)),
        make_cylinder(0.045, 0.052, 0.26, 10, (0.0, -0.13, 0.0)),
    ])
    mesh_upper_arm = b.add_mesh("UpperArmMesh", [(upper_arm_geo, mat_blazer)])

    forearm_geo = merge_meshes([
        make_uv_sphere(0.042, 0.042, 0.042, 8, 10, (0.0, 0.0, 0.0)),
        make_cylinder(0.036, 0.044, 0.22, 10, (0.0, -0.11, 0.0)),
    ])
    hand_geo = make_uv_sphere(0.038, 0.048, 0.032, 8, 10, (0.0, -0.25, 0.01))
    mesh_forearm = b.add_mesh("ForearmMesh", [
        (forearm_geo, mat_blazer),
        (hand_geo, mat_skin),
    ])

    thigh_geo = merge_meshes([
        make_uv_sphere(0.072, 0.072, 0.072, 8, 10, (0.0, 0.0, 0.0)),
        make_cylinder(0.058, 0.072, 0.36, 10, (0.0, -0.18, 0.0)),
    ])
    mesh_thigh = b.add_mesh("ThighMesh", [(thigh_geo, mat_pants)])

    shin_geo = merge_meshes([
        make_uv_sphere(0.056, 0.056, 0.056, 8, 10, (0.0, 0.0, 0.0)),
        make_cylinder(0.046, 0.056, 0.35, 10, (0.0, -0.175, 0.0)),
    ])
    mesh_shin = b.add_mesh("ShinMesh", [(shin_geo, mat_pants)])

    shoe_upper = make_tapered_box(0.095, 0.21, 0.082, 0.16, 0.065, (0.0, -0.045, 0.045))
    shoe_sole = make_box(0.102, 0.025, 0.22, (0.0, -0.085, 0.045))
    mesh_foot = b.add_mesh("FootMesh", [
        (shoe_upper, mat_shoes),
        (shoe_sole, mat_sole),
    ])

    # --- Node Hierarchy ---
    n_foot_l = b.add_node("Foot_L", translation=(0.0, -0.36, 0.0), mesh=mesh_foot)
    n_shin_l = b.add_node("Shin_L", translation=(0.0, -0.37, 0.0), mesh=mesh_shin, children=[n_foot_l])
    n_thigh_l = b.add_node("Thigh_L", translation=(-0.095, -0.03, 0.0), mesh=mesh_thigh, children=[n_shin_l])

    n_foot_r = b.add_node("Foot_R", translation=(0.0, -0.36, 0.0), mesh=mesh_foot)
    n_shin_r = b.add_node("Shin_R", translation=(0.0, -0.37, 0.0), mesh=mesh_shin, children=[n_foot_r])
    n_thigh_r = b.add_node("Thigh_R", translation=(0.095, -0.03, 0.0), mesh=mesh_thigh, children=[n_shin_r])

    n_forearm_l = b.add_node("Forearm_L", translation=(0.0, -0.26, 0.0), mesh=mesh_forearm)
    n_upper_arm_l = b.add_node("UpperArm_L", translation=(-0.225, 0.37, 0.0), mesh=mesh_upper_arm, children=[n_forearm_l])

    n_forearm_r = b.add_node("Forearm_R", translation=(0.0, -0.26, 0.0), mesh=mesh_forearm)
    n_upper_arm_r = b.add_node("UpperArm_R", translation=(0.225, 0.37, 0.0), mesh=mesh_upper_arm, children=[n_forearm_r])

    n_head = b.add_node("Head", translation=(0.0, 0.46, 0.0), mesh=mesh_head)
    n_torso = b.add_node(
        "Torso",
        translation=(0.0, 0.05, 0.0),
        mesh=mesh_torso,
        children=[n_head, n_upper_arm_l, n_upper_arm_r],
    )

    HIP_BASE_Y = 0.8575
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

    # --- Animation 1: Idle ---
    idle_dur = 2.4
    idle_steps = 24
    idle_times = [idle_dur * i / idle_steps for i in range(idle_steps + 1)]
    idle_hips_pos, idle_hips_rot, idle_torso_rot, idle_head_rot = [], [], [], []
    idle_arm_l_rot, idle_arm_r_rot, idle_forearm_l_rot, idle_forearm_r_rot = [], [], [], []

    for i in range(idle_steps + 1):
        p = 2.0 * math.pi * i / idle_steps
        breath = math.sin(p)
        sway = math.sin(p * 0.5)
        idle_hips_pos.append((0.0, HIP_BASE_Y + 0.008 * breath, 0.0))
        idle_hips_rot.append(euler_to_quat(0.0, 0.03 * sway, 0.015 * sway))
        idle_torso_rot.append(euler_to_quat(0.025 * breath, -0.02 * sway, -0.01 * sway))
        idle_head_rot.append(euler_to_quat(-0.02 * breath + 0.02 * math.sin(p), 0.05 * math.sin(p), 0.02 * sway))
        idle_arm_l_rot.append(euler_to_quat(0.04 * breath, 0.0, -0.09 - 0.015 * breath))
        idle_arm_r_rot.append(euler_to_quat(0.04 * breath, 0.0, 0.09 + 0.015 * breath))
        idle_forearm_l_rot.append(euler_to_quat(-0.22 - 0.04 * breath, 0.0, 0.0))
        idle_forearm_r_rot.append(euler_to_quat(-0.22 - 0.04 * breath, 0.0, 0.0))

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

    # --- Animation 2: Walk ---
    walk_dur = 0.85
    walk_steps = 24
    walk_times = [walk_dur * i / walk_steps for i in range(walk_steps + 1)]
    w_hips_pos, w_hips_rot, w_torso_rot, w_head_rot = [], [], [], []
    w_thigh_l, w_shin_l, w_foot_l = [], [], []
    w_thigh_r, w_shin_r, w_foot_r = [], [], []
    w_uarm_l, w_farm_l, w_uarm_r, w_farm_r = [], [], [], []

    for i in range(walk_steps + 1):
        p = 2.0 * math.pi * i / walk_steps
        bob = -0.028 * abs(math.sin(p)) + 0.012
        sway_x = 0.018 * math.sin(p)
        w_hips_pos.append((sway_x, HIP_BASE_Y + bob, 0.0))
        w_hips_rot.append(euler_to_quat(0.03, 0.10 * math.sin(p), -0.04 * math.sin(p)))
        w_torso_rot.append(euler_to_quat(0.07, -0.14 * math.sin(p), 0.03 * math.sin(p)))
        w_head_rot.append(euler_to_quat(-0.04, 0.05 * math.sin(p), 0.0))

        for phase, thigh_arr, shin_arr, foot_arr in [
            (p, w_thigh_l, w_shin_l, w_foot_l),
            (p + math.pi, w_thigh_r, w_shin_r, w_foot_r),
        ]:
            thigh_rx = -0.55 * math.sin(phase)
            knee_rx = max(0.05, 0.70 * max(0.0, math.cos(phase)) + 0.18 * (1.0 - math.sin(phase)) * 0.3)
            foot_rx = -0.20 * math.sin(phase) - 0.15 * max(0.0, -math.cos(phase))
            thigh_arr.append(euler_to_quat(thigh_rx, 0.0, 0.0))
            shin_arr.append(euler_to_quat(knee_rx, 0.0, 0.0))
            foot_arr.append(euler_to_quat(foot_rx, 0.0, 0.0))

        w_uarm_l.append(euler_to_quat(0.42 * math.sin(p), 0.0, -0.08))
        w_uarm_r.append(euler_to_quat(-0.42 * math.sin(p), 0.0, 0.08))
        w_farm_l.append(euler_to_quat(-0.45 - 0.22 * max(0.0, -math.sin(p)), 0.0, 0.0))
        w_farm_r.append(euler_to_quat(-0.45 - 0.22 * max(0.0, math.sin(p)), 0.0, 0.0))

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

    b.build_glb(out_path, root_nodes=[n_root])


if __name__ == "__main__":
    output = os.path.join("assets", "characters", "player", "character_founder.glb")
    build_character_glb(output)
    print(f"Generated {output} ({os.path.getsize(output)} bytes)")
