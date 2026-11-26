use bevy::prelude::*;

use crate::{
    player::{CameraMode, PlayerCamera, PlayerController},
    ui::WorkstationUiState,
};

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveInteractionTarget>()
            .add_message::<InteractTriggered>()
            .add_systems(
                Update,
                (update_interaction_target, trigger_interaction).chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractableKind {
    Door,
    Workstation,
    Courier,
    Employee,
    Sofa,
}

#[derive(Component)]
pub struct Interactable {
    pub kind: InteractableKind,
    pub prompt: String,
    pub range: f32,
    pub offset: Vec3,
}

#[derive(Resource, Default)]
pub struct ActiveInteractionTarget {
    pub entity: Option<Entity>,
    pub kind: Option<InteractableKind>,
    pub prompt: Option<String>,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct InteractTriggered {
    pub entity: Entity,
    pub kind: InteractableKind,
}

fn update_interaction_target(
    ui_state: Res<WorkstationUiState>,
    player_q: Query<(&Transform, &PlayerController)>,
    cam_q: Query<&Transform, With<PlayerCamera>>,
    interactables: Query<(Entity, &Transform, &Interactable)>,
    mut active: ResMut<ActiveInteractionTarget>,
) {
    if ui_state.is_open {
        active.entity = None;
        active.kind = None;
        active.prompt = None;
        return;
    }

    let Ok((player_tf, ctrl)) = player_q.single() else {
        return;
    };
    let Ok(cam_tf) = cam_q.single() else {
        return;
    };

    let player_chest = player_tf.translation + Vec3::new(0.0, 1.05, 0.0);
    let cam_forward = cam_tf.forward().as_vec3();
    let body_forward = player_tf.forward().as_vec3();

    let mut best: Option<(f32, Entity, InteractableKind, String)> = None;

    for (entity, tf, item) in &interactables {
        let target_pos = tf.translation + item.offset;
        let dist = player_chest.distance(target_pos);
        if dist > item.range {
            continue;
        }

        let (aim_dot, min_dot) = match ctrl.camera_mode {
            CameraMode::FirstPerson => {
                let to_target = (target_pos - cam_tf.translation).normalize_or_zero();
                (cam_forward.dot(to_target), 0.72)
            }
            CameraMode::ThirdPerson => {
                let mut to_target_xz = target_pos - player_chest;
                to_target_xz.y = 0.0;
                let dir_xz = to_target_xz.normalize_or_zero();
                let body_xz = Vec3::new(body_forward.x, 0.0, body_forward.z).normalize_or_zero();
                let cam_xz = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();
                (body_xz.dot(dir_xz).max(cam_xz.dot(dir_xz)), 0.30)
            }
        };

        if aim_dot < min_dot {
            continue;
        }

        // Higher score prefers closer & better-aligned objects
        let score = aim_dot * 2.0 - (dist / item.range);
        if best.as_ref().is_none_or(|b| score > b.0) {
            best = Some((score, entity, item.kind, item.prompt.clone()));
        }
    }

    if let Some((_, entity, kind, prompt)) = best {
        active.entity = Some(entity);
        active.kind = Some(kind);
        active.prompt = Some(prompt);
    } else {
        active.entity = None;
        active.kind = None;
        active.prompt = None;
    }
}

fn trigger_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    ui_state: Res<WorkstationUiState>,
    active: Res<ActiveInteractionTarget>,
    mut writer: MessageWriter<InteractTriggered>,
) {
    if ui_state.is_open || ui_state.is_changed() || !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    if let (Some(entity), Some(kind)) = (active.entity, active.kind) {
        writer.write(InteractTriggered { entity, kind });
    }
}
