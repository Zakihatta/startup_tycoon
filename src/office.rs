use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

use crate::player::PlayerController;

pub struct OfficePlugin;

impl Plugin for OfficePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OfficeColliders>()
            .add_systems(Startup, setup_office_scene)
            .add_systems(Update, (toggle_door_interaction, animate_door));
    }
}

#[derive(Clone, Copy)]
pub struct Aabb2d {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

#[derive(Resource, Default)]
pub struct OfficeColliders {
    pub obstacles: Vec<Aabb2d>,
    pub room_half_x: f32,
    pub room_half_z: f32,
}

#[derive(Component)]
pub struct OfficeDoor {
    pub is_open: bool,
    pub current_angle: f32,
    pub target_angle: f32,
    pub hinge_pos: Vec3,
}

fn setup_office_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut colliders: ResMut<OfficeColliders>,
) {
    // 6m x 5m small startup office centered at (0, 0, 0)
    // Grid tiles: x in [-2.5..=2.5], z in [-2.0..=2.0]
    let floor_handle = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("environment/floor/environment_floor_tile.glb"),
    );
    let wall_plain = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("environment/walls/environment_wall_plain.glb"),
    );
    let wall_window = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("environment/walls/environment_wall_window.glb"),
    );
    let wall_door = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("environment/walls/environment_wall_door.glb"),
    );
    let door_handle = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("environment/doors/environment_door_basic.glb"),
    );

    // Spawn 6 x 5 modular floor tiles
    for gx in 0..6 {
        for gz in 0..5 {
            let x = -2.5 + gx as f32;
            let z = -2.0 + gz as f32;
            commands.spawn((
                WorldAssetRoot(floor_handle.clone()),
                Transform::from_xyz(x, 0.0, z),
            ));
        }
    }

    // North wall (z = -2.5): 6 segments with windows in the middle
    for gx in 0..6 {
        let x = -2.5 + gx as f32;
        let scene = if gx == 1 || gx == 2 || gx == 4 {
            wall_window.clone()
        } else {
            wall_plain.clone()
        };
        commands.spawn((
            WorldAssetRoot(scene),
            Transform::from_xyz(x, 0.0, -2.5),
        ));
    }

    // South wall (z = +2.5): door module at gx = 3 (x = +0.5)
    for gx in 0..6 {
        let x = -2.5 + gx as f32;
        if gx == 3 {
            commands.spawn((
                WorldAssetRoot(wall_door.clone()),
                Transform::from_xyz(x, 0.0, 2.5),
            ));
            // Door leaf hinged at left side of opening (x = 0.5 - 0.40 = 0.10)
            let hinge_pos = Vec3::new(x - 0.40, 0.0, 2.5);
            commands.spawn((
                WorldAssetRoot(door_handle.clone()),
                Transform::from_translation(hinge_pos),
                OfficeDoor {
                    is_open: false,
                    current_angle: 0.0,
                    target_angle: 0.0,
                    hinge_pos,
                },
            ));
        } else {
            commands.spawn((
                WorldAssetRoot(wall_plain.clone()),
                Transform::from_xyz(x, 0.0, 2.5),
            ));
        }
    }

    // West wall (x = -3.0) & East wall (x = +3.0): 5 segments each, rotated 90 deg
    for gz in 0..5 {
        let z = -2.0 + gz as f32;
        commands.spawn((
            WorldAssetRoot(wall_plain.clone()),
            Transform::from_xyz(-3.0, 0.0, z).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        ));
        let east_scene = if gz == 1 || gz == 2 {
            wall_window.clone()
        } else {
            wall_plain.clone()
        };
        commands.spawn((
            WorldAssetRoot(east_scene),
            Transform::from_xyz(3.0, 0.0, z).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        ));
    }

    // Initial Workstation (M2 preview inside the M1 office)
    let desk_pos = Vec3::new(-1.3, 0.0, -1.65);
    commands.spawn((
        WorldAssetRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("furniture/furniture_desk_basic.glb"),
            ),
        ),
        Transform::from_translation(desk_pos),
    ));

    // Office chair in front of desk (+Z side), facing -Z toward the desk
    commands.spawn((
        WorldAssetRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("furniture/furniture_chair_office.glb"),
            ),
        ),
        Transform::from_xyz(-1.3, 0.0, -0.82),
    ));

    // Monitor, keyboard/mouse, and PC on top of the desk (y = 0.75)
    commands.spawn((
        WorldAssetRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("equipment/equipment_monitor_basic.glb"),
            ),
        ),
        Transform::from_xyz(-1.3, 0.75, -1.82),
    ));
    commands.spawn((
        WorldAssetRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("equipment/equipment_keyboard_mouse.glb"),
            ),
        ),
        Transform::from_xyz(-1.3, 0.75, -1.52),
    ));
    commands.spawn((
        WorldAssetRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("equipment/equipment_pc_basic.glb"),
            ),
        ),
        Transform::from_xyz(-0.78, 0.75, -1.72),
    ));

    // Lighting: warm daylight through windows + soft indoor ceiling light
    commands.spawn((
        DirectionalLight {
            illuminance: 9_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 8.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.96, 0.88),
            intensity: 180_000.0,
            range: 14.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 2.55, 0.0),
    ));

    // Register room bounds & solid furniture colliders (half-extents with margin)
    colliders.room_half_x = 2.68;
    colliders.room_half_z = 2.18;
    colliders.obstacles = vec![
        // Desk + chair area
        Aabb2d {
            min_x: -2.05,
            max_x: -0.55,
            min_z: -2.05,
            max_z: -0.60,
        },
    ];
}

fn toggle_door_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    player_q: Query<&Transform, With<PlayerController>>,
    mut door_q: Query<&mut OfficeDoor>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_chest = player_tf.translation + Vec3::new(0.0, 1.0, 0.0);

    for mut door in &mut door_q {
        let door_center = door.hinge_pos + Vec3::new(0.40, 1.0, 0.0);
        if player_chest.distance(door_center) <= 2.4 {
            door.is_open = !door.is_open;
            door.target_angle = if door.is_open { -1.45 } else { 0.0 };
        }
    }
}

fn animate_door(time: Res<Time>, mut door_q: Query<(&mut Transform, &mut OfficeDoor)>) {
    let dt = time.delta_secs();
    for (mut tf, mut door) in &mut door_q {
        let diff = door.target_angle - door.current_angle;
        if diff.abs() > 0.001 {
            door.current_angle += diff * (8.0 * dt).min(1.0);
            tf.rotation = Quat::from_rotation_y(door.current_angle);
        }
    }
}
