use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui::Pressed,
    ui_widgets::{Activate, Button},
    window::{CursorGrabMode, CursorOptions},
};

use crate::{
    interaction::{ActiveInteractionTarget, InteractTriggered, InteractableKind},
    player::{CameraMode, PlayerController},
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorkstationUiState>()
            .add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    handle_workstation_close,
                    handle_workstation_interact,
                    update_hud_mode,
                    update_interaction_prompt,
                    sync_workstation_modal_visibility,
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Default)]
pub struct WorkstationUiState {
    pub is_open: bool,
}

#[derive(Component)]
struct HudHintText;

#[derive(Component)]
struct CrosshairDot;

#[derive(Component)]
struct InteractionPromptBox;

#[derive(Component)]
struct InteractionPromptText;

#[derive(Component)]
struct WorkstationModal;

#[derive(Component)]
struct CloseWorkstationButton;

fn setup_ui(mut commands: Commands) {
    // Top-left controls & mode overlay
    commands.spawn((
        HudHintText,
        Text::new(
            "Startup Tycoon (M2 Prototype) - Mode: 3rd Person\n\
             [Klik Kiri] Kunci Mouse | [ESC] Lepas Mouse\n\
             [WASD] Jalan | [Shift] Lari | [V] 1st/3rd Person | [E] Interaksi",
        ),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.94)),
        BackgroundColor(Color::srgba(0.08, 0.10, 0.14, 0.70)),
        Node {
            position_type: PositionType::Absolute,
            top: px(14.0),
            left: px(14.0),
            padding: UiRect::all(px(10.0)),
            ..default()
        },
    ));

    // Center crosshair dot
    commands
        .spawn((
            CrosshairDot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: px(6.0),
                    height: px(6.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
            ));
        });

    // Dynamic interaction prompt below center
    commands
        .spawn((
            InteractionPromptBox,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                top: percent(56.0),
                width: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    BackgroundColor(Color::srgba(0.07, 0.12, 0.18, 0.86)),
                    Node {
                        padding: UiRect::axes(px(14.0), px(8.0)),
                        border: UiRect::all(px(1.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.20, 0.72, 0.54)),
                ))
                .with_children(|box_parent| {
                    box_parent.spawn((
                        InteractionPromptText,
                        Text::new("[E] Interaksi"),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.98, 1.0)),
                    ));
                });
        });

    // Workstation Computer Modal (hidden by default)
    commands
        .spawn((
            WorkstationModal,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.05, 0.09, 0.65)),
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn((
                    Node {
                        width: px(620.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(px(22.0)),
                        row_gap: px(14.0),
                        border: UiRect::all(px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.10, 0.13, 0.19)),
                    BorderColor::all(Color::srgb(0.20, 0.72, 0.54)),
                ))
                .with_children(|panel| {
                    // Header row
                    panel
                        .spawn(Node {
                            width: percent(100.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|header| {
                            header.spawn((
                                Text::new("StartupOS v0.1 - Workstation"),
                                TextFont {
                                    font_size: FontSize::Px(22.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.24, 0.84, 0.62)),
                            ));

                            header
                                .spawn((
                                    CloseWorkstationButton,
                                    Button,
                                    Hovered::default(),
                                    Node {
                                        padding: UiRect::axes(px(12.0), px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.24, 0.28, 0.36)),
                                ))
                                .observe(
                                    |_: On<Activate>,
                                     mut ui_state: ResMut<WorkstationUiState>| {
                                        ui_state.is_open = false;
                                    },
                                )
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Tutup [ESC / E]"),
                                        TextFont {
                                            font_size: FontSize::Px(14.0),
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });

                    // Body status summary (baseline GDD M3 preview)
                    panel.spawn((
                        Text::new(
                            "Status Perusahaan (Baseline M2)\n\
                             • Nama Perusahaan : Startup Kamu (Teknologi)\n\
                             • Hari Simulasi   : Day 1\n\
                             • Modal Kas Awal  : $25,000\n\
                             • Produk Aktif    : Belum ada (dibuka di M4)\n\
                             • Tim             : Founder (1)",
                        ),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.90, 0.93, 0.97)),
                        Node {
                            padding: UiRect::all(px(14.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.14, 0.18, 0.25)),
                    ));

                    panel.spawn((
                        Text::new(
                            "Menu Manajemen (Finance, Product Development, Hiring, Market) \
                             akan terhubung ke layar ini pada Milestone M3–M7.",
                        ),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.70, 0.76, 0.84)),
                    ));
                });
        });
}

fn handle_workstation_interact(
    mut reader: MessageReader<InteractTriggered>,
    mut ui_state: ResMut<WorkstationUiState>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    for ev in reader.read() {
        if ev.kind == InteractableKind::Workstation {
            ui_state.is_open = true;
            cursor_options.visible = true;
            cursor_options.grab_mode = CursorGrabMode::None;
        }
    }
}

fn handle_workstation_close(
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<WorkstationUiState>,
    mut btn_q: Query<
        (&Hovered, Option<&Pressed>, &mut BackgroundColor),
        With<CloseWorkstationButton>,
    >,
) {
    if !ui_state.is_open || ui_state.is_changed() {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyE) {
        ui_state.is_open = false;
        return;
    }

    for (hovered, pressed, mut bg) in &mut btn_q {
        if pressed.is_some() {
            *bg = BackgroundColor(Color::srgb(0.18, 0.56, 0.42));
        } else if hovered.get() {
            *bg = BackgroundColor(Color::srgb(0.32, 0.38, 0.48));
        } else {
            *bg = BackgroundColor(Color::srgb(0.24, 0.28, 0.36));
        }
    }
}

fn update_hud_mode(
    player_q: Query<&PlayerController, Changed<PlayerController>>,
    mut text_q: Query<&mut Text, With<HudHintText>>,
) {
    let Ok(ctrl) = player_q.single() else {
        return;
    };
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };
    let mode_label = match ctrl.camera_mode {
        CameraMode::FirstPerson => "1st Person",
        CameraMode::ThirdPerson => "3rd Person",
    };
    text.0 = format!(
        "Startup Tycoon (M2 Prototype) - Mode: {mode_label}\n\
         [Klik Kiri] Kunci Mouse | [ESC] Lepas Mouse\n\
         [WASD] Jalan | [Shift] Lari | [V] 1st/3rd Person | [E] Interaksi"
    );
}

fn update_interaction_prompt(
    active: Res<ActiveInteractionTarget>,
    ui_state: Res<WorkstationUiState>,
    mut box_q: Query<&mut Visibility, With<InteractionPromptBox>>,
    mut text_q: Query<&mut Text, With<InteractionPromptText>>,
) {
    let Ok(mut vis) = box_q.single_mut() else {
        return;
    };
    if ui_state.is_open {
        *vis = Visibility::Hidden;
        return;
    }

    if let Some(prompt) = &active.prompt {
        *vis = Visibility::Visible;
        if let Ok(mut text) = text_q.single_mut() {
            text.0 = format!("[E] {prompt}");
        }
    } else {
        *vis = Visibility::Hidden;
    }
}

fn sync_workstation_modal_visibility(
    ui_state: Res<WorkstationUiState>,
    mut modal_q: Query<&mut Visibility, (With<WorkstationModal>, Without<CrosshairDot>)>,
    mut crosshair_q: Query<&mut Visibility, (With<CrosshairDot>, Without<WorkstationModal>)>,
) {
    if !ui_state.is_changed() {
        return;
    }
    if let Ok(mut modal_vis) = modal_q.single_mut() {
        *modal_vis = if ui_state.is_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut cross_vis) = crosshair_q.single_mut() {
        *cross_vis = if ui_state.is_open {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}
