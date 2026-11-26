use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use std::{
    f32::consts::{FRAC_PI_2, PI},
    time::Duration,
};

use crate::{office::OfficeColliders, ui::WorkstationUiState};

const FOUNDER_ASSET: &str = "characters/player/character_founder.glb";

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    toggle_cursor_grab,
                    toggle_camera_mode,
                    player_look,
                    player_move,
                    setup_founder_animations,
                    sync_founder_animations,
                    update_player_camera,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
}

#[derive(Component)]
pub struct PlayerController {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub walk_speed: f32,
    pub sprint_speed: f32,
    pub radius: f32,
    pub camera_mode: CameraMode,
    pub is_moving: bool,
    pub is_sprinting: bool,
    pub facing_yaw: f32,
}

#[derive(Component)]
pub struct PlayerVisual;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Resource)]
struct FounderAnimations {
    graph: Handle<AnimationGraph>,
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    last_moving: Option<bool>,
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let idle_clip = asset_server.load(GltfAssetLabel::Animation(0).from_asset(FOUNDER_ASSET));
    let walk_clip = asset_server.load(GltfAssetLabel::Animation(1).from_asset(FOUNDER_ASSET));

    let (graph, nodes) = AnimationGraph::from_clips([idle_clip, walk_clip]);
    let graph_handle = graphs.add(graph);

    commands.insert_resource(FounderAnimations {
        graph: graph_handle,
        idle: nodes[0],
        walk: nodes[1],
        last_moving: None,
    });

    let initial_yaw = 0.25;
    let initial_pitch = -0.18;

    // Playable Founder body at ground y = 0
    commands
        .spawn((
            Transform::from_xyz(0.8, 0.0, 1.1).with_rotation(Quat::from_rotation_y(initial_yaw)),
            Visibility::default(),
            PlayerController {
                yaw: initial_yaw,
                pitch: initial_pitch,
                sensitivity: 0.0022,
                walk_speed: 3.0,
                sprint_speed: 5.0,
                radius: 0.28,
                camera_mode: CameraMode::ThirdPerson,
                is_moving: false,
                is_sprinting: false,
                facing_yaw: initial_yaw,
            },
            PlayerVisual,
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(FOUNDER_ASSET))),
        ));

    // Dedicated camera entity synced to PlayerController
    commands.spawn((Camera3d::default(), Transform::default(), PlayerCamera));
}

fn toggle_cursor_grab(
    ui_state: Res<WorkstationUiState>,
    mut cursor_options: Single<&mut CursorOptions>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if ui_state.is_open {
        return;
    }
    if mouse.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
    if keys.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
}

fn toggle_camera_mode(
    keys: Res<ButtonInput<KeyCode>>,
    ui_state: Res<WorkstationUiState>,
    mut player_q: Query<(&mut PlayerController, &mut Visibility), With<PlayerVisual>>,
) {
    if ui_state.is_open || !keys.just_pressed(KeyCode::KeyV) {
        return;
    }
    let Ok((mut player, mut vis)) = player_q.single_mut() else {
        return;
    };

    player.camera_mode = match player.camera_mode {
        CameraMode::FirstPerson => {
            *vis = Visibility::Inherited;
            player.pitch = player.pitch.clamp(-1.1, 0.5);
            CameraMode::ThirdPerson
        }
        CameraMode::ThirdPerson => {
            *vis = Visibility::Hidden;
            player.facing_yaw = player.yaw;
            CameraMode::FirstPerson
        }
    };
}

fn player_look(
    ui_state: Res<WorkstationUiState>,
    cursor_options: Single<&CursorOptions>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut player_q: Query<&mut PlayerController>,
) {
    if ui_state.is_open || cursor_options.grab_mode == CursorGrabMode::None {
        return;
    }
    let delta = mouse_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }
    let Ok(mut player) = player_q.single_mut() else {
        return;
    };

    player.yaw -= delta.x * player.sensitivity;
    let (min_pitch, max_pitch) = match player.camera_mode {
        CameraMode::FirstPerson => (-(FRAC_PI_2 - 0.05), FRAC_PI_2 - 0.05),
        CameraMode::ThirdPerson => (-1.15, 0.45),
    };
    player.pitch = (player.pitch - delta.y * player.sensitivity).clamp(min_pitch, max_pitch);
}

fn shortest_angle_diff(from: f32, to: f32) -> f32 {
    let mut diff = (to - from) % (2.0 * PI);
    if diff > PI {
        diff -= 2.0 * PI;
    } else if diff < -PI {
        diff += 2.0 * PI;
    }
    diff
}

fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    ui_state: Res<WorkstationUiState>,
    colliders: Res<OfficeColliders>,
    mut player_q: Query<(&mut Transform, &mut PlayerController)>,
) {
    let Ok((mut tf, mut player)) = player_q.single_mut() else {
        return;
    };

    if ui_state.is_open {
        player.is_moving = false;
        player.is_sprinting = false;
        return;
    }

    // Derive horizontal forward/right from camera yaw
    let yaw_rot = Quat::from_rotation_y(player.yaw);
    let flat_forward = yaw_rot * Vec3::NEG_Z;
    let flat_right = yaw_rot * Vec3::X;

    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        dir += flat_forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir -= flat_forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir += flat_right;
    }
    if keys.pressed(KeyCode::KeyA) {
        dir -= flat_right;
    }

    if dir == Vec3::ZERO {
        player.is_moving = false;
        player.is_sprinting = false;
        if player.camera_mode == CameraMode::FirstPerson {
            player.facing_yaw = player.yaw;
            tf.rotation = Quat::from_rotation_y(player.facing_yaw);
        }
        return;
    }

    let move_dir = dir.normalize();
    let sprinting = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let speed = if sprinting {
        player.sprint_speed
    } else {
        player.walk_speed
    };

    player.is_moving = true;
    player.is_sprinting = sprinting;

    let dt = time.delta_secs();
    let step = move_dir * speed * dt;
    let r = player.radius;

    // Axis-separated collision for smooth wall sliding
    let mut next_x = (tf.translation.x + step.x).clamp(-colliders.room_half_x, colliders.room_half_x);
    for box2d in &colliders.obstacles {
        if tf.translation.z > box2d.min_z - r && tf.translation.z < box2d.max_z + r
            && next_x > box2d.min_x - r
            && next_x < box2d.max_x + r
        {
            next_x = tf.translation.x;
        }
    }
    tf.translation.x = next_x;

    let mut next_z = (tf.translation.z + step.z).clamp(-colliders.room_half_z, colliders.room_half_z);
    for box2d in &colliders.obstacles {
        if tf.translation.x > box2d.min_x - r && tf.translation.x < box2d.max_x + r
            && next_z > box2d.min_z - r
            && next_z < box2d.max_z + r
        {
            next_z = tf.translation.z;
        }
    }
    tf.translation.z = next_z;

    // Rotate character mesh smoothly toward movement direction (-Z forward)
    let target_yaw = (-move_dir.x).atan2(-move_dir.z);
    let diff = shortest_angle_diff(player.facing_yaw, target_yaw);
    player.facing_yaw += diff * (12.0 * dt).min(1.0);
    tf.rotation = Quat::from_rotation_y(player.facing_yaw);
}

fn setup_founder_animations(
    mut commands: Commands,
    mut anims: ResMut<FounderAnimations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, anims.idle, Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(anims.graph.clone()))
            .insert(transitions);

        anims.last_moving = Some(false);
    }
}

fn sync_founder_animations(
    player_q: Query<&PlayerController>,
    mut anims: ResMut<FounderAnimations>,
    mut anim_q: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    let Ok(ctrl) = player_q.single() else {
        return;
    };
    let Ok((mut anim_player, mut transitions)) = anim_q.single_mut() else {
        return;
    };

    if anims.last_moving != Some(ctrl.is_moving) {
        let target = if ctrl.is_moving {
            anims.walk
        } else {
            anims.idle
        };
        transitions
            .play(&mut anim_player, target, Duration::from_millis(160))
            .repeat();
        anims.last_moving = Some(ctrl.is_moving);
    }

    // Scale walk speed when sprinting
    let target_speed = if ctrl.is_moving && ctrl.is_sprinting {
        1.5
    } else {
        1.0
    };
    if let Some(active) = anim_player.playing_animations_mut().next() {
        active.1.set_speed(target_speed);
    }
}

fn update_player_camera(
    player_q: Query<(&Transform, &PlayerController), Without<PlayerCamera>>,
    mut cam_q: Query<&mut Transform, With<PlayerCamera>>,
) {
    let Ok((player_tf, ctrl)) = player_q.single() else {
        return;
    };
    let Ok(mut cam_tf) = cam_q.single_mut() else {
        return;
    };

    let cam_rot = Quat::from_rotation_y(ctrl.yaw) * Quat::from_rotation_x(ctrl.pitch);

    match ctrl.camera_mode {
        CameraMode::FirstPerson => {
            cam_tf.translation = player_tf.translation + Vec3::new(0.0, 1.65, 0.0);
            cam_tf.rotation = cam_rot;
        }
        CameraMode::ThirdPerson => {
            let pivot = player_tf.translation + Vec3::new(0.0, 1.38, 0.0);
            let back_dir = cam_rot * Vec3::Z;
            let desired = pivot + back_dir * 2.4;

            // Keep camera inside room bounds so walls don't block the view
            cam_tf.translation = Vec3::new(
                desired.x.clamp(-2.82, 2.82),
                desired.y.clamp(0.35, 2.65),
                desired.z.clamp(-2.32, 2.32),
            );
            cam_tf.look_at(pivot, Vec3::Y);
        }
    }
}
