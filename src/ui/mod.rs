use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui::Pressed,
    ui_widgets::{Activate, Button},
    window::{CursorGrabMode, CursorOptions},
};

use crate::{
    company::CompanyState,
    core::{GameClock, SimSpeed},
    economy::FinanceState,
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
                    update_hud_status,
                    update_speed_button_styles,
                    update_tab_button_styles,
                    update_interaction_prompt,
                    sync_workstation_modal_visibility,
                    update_workstation_contents,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkstationTab {
    #[default]
    Overview,
    Finance,
}

#[derive(Resource, Default)]
pub struct WorkstationUiState {
    pub is_open: bool,
    pub active_tab: WorkstationTab,
}

/// Marker for HUD buttons so clicking them doesn't lock the mouse cursor.
#[derive(Component)]
pub struct HudInteractive;

#[derive(Component)]
struct HudHintText;

#[derive(Component)]
struct HudCashText;

#[derive(Component)]
struct HudTimeText;

#[derive(Component, Clone, Copy)]
struct SpeedButton(SimSpeed);

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

#[derive(Component, Clone, Copy)]
struct TabButton(WorkstationTab);

#[derive(Component)]
struct OverviewBodyText;

#[derive(Component)]
struct FinanceBodyText;

fn format_money(amount: i64) -> String {
    let neg = amount < 0;
    let digits = amount.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let formatted: String = out.chars().rev().collect();
    if neg {
        format!("-${formatted}")
    } else {
        format!("${formatted}")
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon_money: Handle<Image> = asset_server.load("ui/icons/ui_icon_money.png");
    let icon_time: Handle<Image> = asset_server.load("ui/icons/ui_icon_time.png");
    let icon_company: Handle<Image> = asset_server.load("ui/icons/ui_icon_company.png");

    // Top-left controls & mode overlay
    commands.spawn((
        HudHintText,
        Text::new(
            "Startup Tycoon (M3) - Mode: 3rd Person\n\
             [Klik Kiri] Kunci Mouse | [ESC] Lepas Mouse\n\
             [WASD] Jalan | [Shift] Lari | [V] 1st/3rd Person | [E] Interaksi\n\
             [0/P] Pause | [1] 1x | [2] 2x",
        ),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.94)),
        BackgroundColor(Color::srgba(0.08, 0.10, 0.14, 0.72)),
        Node {
            position_type: PositionType::Absolute,
            top: px(14.0),
            left: px(14.0),
            padding: UiRect::all(px(10.0)),
            ..default()
        },
    ));

    // Top-right Cash & Time HUD bar
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(14.0),
                right: px(14.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(12.0),
                padding: UiRect::axes(px(14.0), px(9.0)),
                border: UiRect::all(px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.11, 0.16, 0.85)),
            BorderColor::all(Color::srgb(0.22, 0.30, 0.42)),
        ))
        .with_children(|bar| {
            // Cash group
            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(6.0),
                ..default()
            })
            .with_children(|cash| {
                cash.spawn((
                    ImageNode::new(icon_money),
                    Node {
                        width: px(20.0),
                        height: px(20.0),
                        ..default()
                    },
                ));
                cash.spawn((
                    HudCashText,
                    Text::new("$25,000"),
                    TextFont {
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.24, 0.86, 0.62)),
                ));
            });

            // Time group
            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(6.0),
                ..default()
            })
            .with_children(|time_box| {
                time_box.spawn((
                    ImageNode::new(icon_time),
                    Node {
                        width: px(20.0),
                        height: px(20.0),
                        ..default()
                    },
                ));
                time_box.spawn((
                    HudTimeText,
                    Text::new("Hari 1 · 09:00"),
                    TextFont {
                        font_size: FontSize::Px(15.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.95, 1.0)),
                ));
            });

            // Speed buttons
            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(4.0),
                ..default()
            })
            .with_children(|btns| {
                for (speed, label) in [
                    (SimSpeed::Paused, "||"),
                    (SimSpeed::Normal, "1x"),
                    (SimSpeed::Fast, "2x"),
                ] {
                    btns.spawn((
                        SpeedButton(speed),
                        HudInteractive,
                        Button,
                        Hovered::default(),
                        Node {
                            padding: UiRect::axes(px(8.0), px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.23, 0.31)),
                    ))
                    .observe(move |_: On<Activate>, mut clock: ResMut<GameClock>| {
                        clock.speed = speed;
                    })
                    .with_children(|b| {
                        b.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: FontSize::Px(13.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });
        });

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

    // Workstation Computer Modal (StartupOS)
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
            BackgroundColor(Color::srgba(0.03, 0.05, 0.09, 0.68)),
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn((
                    Node {
                        width: px(700.0),
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
                            header
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: px(10.0),
                                    ..default()
                                })
                                .with_children(|title_row| {
                                    title_row.spawn((
                                        ImageNode::new(icon_company),
                                        Node {
                                            width: px(24.0),
                                            height: px(24.0),
                                            ..default()
                                        },
                                    ));
                                    title_row.spawn((
                                        Text::new("StartupOS v0.2 - Manajemen Perusahaan"),
                                        TextFont {
                                            font_size: FontSize::Px(21.0),
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.24, 0.84, 0.62)),
                                    ));
                                });

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

                    // Navigation tabs
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(8.0),
                            ..default()
                        })
                        .with_children(|tabs| {
                            for (tab, label) in [
                                (WorkstationTab::Overview, "Overview"),
                                (WorkstationTab::Finance, "Finance"),
                            ] {
                                tabs.spawn((
                                    TabButton(tab),
                                    Button,
                                    Hovered::default(),
                                    Node {
                                        padding: UiRect::axes(px(14.0), px(7.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.18, 0.23, 0.31)),
                                ))
                                .observe(
                                    move |_: On<Activate>,
                                          mut ui_state: ResMut<WorkstationUiState>| {
                                        ui_state.active_tab = tab;
                                    },
                                )
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new(label),
                                        TextFont {
                                            font_size: FontSize::Px(15.0),
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            }
                        });

                    // Overview Body
                    panel.spawn((
                        OverviewBodyText,
                        Text::new("Loading company overview..."),
                        TextFont {
                            font_size: FontSize::Px(15.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.94, 0.98)),
                        Node {
                            padding: UiRect::all(px(14.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.14, 0.18, 0.25)),
                    ));

                    // Finance Body
                    panel.spawn((
                        FinanceBodyText,
                        Visibility::Hidden,
                        Text::new("Loading finance report..."),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.94, 0.98)),
                        Node {
                            display: Display::None,
                            padding: UiRect::all(px(14.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.14, 0.18, 0.25)),
                    ));

                    panel.spawn((
                        Text::new(
                            "Tips: Gunakan [0/P] Pause, [1] 1x, [2] 2x untuk menguji arus kas & sewa kantor harian. \
                             Menu Product & Employee akan hadir di M4–M5.",
                        ),
                        TextFont {
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.68, 0.74, 0.82)),
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
        "Startup Tycoon (M3) - Mode: {mode_label}\n\
         [Klik Kiri] Kunci Mouse | [ESC] Lepas Mouse\n\
         [WASD] Jalan | [Shift] Lari | [V] 1st/3rd Person | [E] Interaksi\n\
         [0/P] Pause | [1] 1x | [2] 2x"
    );
}

fn update_hud_status(
    clock: Res<GameClock>,
    finance: Res<FinanceState>,
    mut cash_q: Query<(&mut Text, &mut TextColor), (With<HudCashText>, Without<HudTimeText>)>,
    mut time_q: Query<&mut Text, (With<HudTimeText>, Without<HudCashText>)>,
) {
    if !clock.is_changed() && !finance.is_changed() {
        return;
    }
    if let Ok((mut cash_text, mut cash_color)) = cash_q.single_mut() {
        cash_text.0 = format_money(finance.cash_balance);
        *cash_color = if finance.is_low_cash() {
            TextColor(Color::srgb(0.96, 0.38, 0.38))
        } else {
            TextColor(Color::srgb(0.24, 0.86, 0.62))
        };
    }
    if let Ok(mut time_text) = time_q.single_mut() {
        time_text.0 = format!(
            "Hari {} · {} [{}]",
            clock.day,
            clock.formatted_time(),
            clock.speed.label()
        );
    }
}

fn update_speed_button_styles(
    clock: Res<GameClock>,
    mut btn_q: Query<(&SpeedButton, &Hovered, Option<&Pressed>, &mut BackgroundColor)>,
) {
    for (btn, hovered, pressed, mut bg) in &mut btn_q {
        if pressed.is_some() || clock.speed == btn.0 {
            *bg = BackgroundColor(Color::srgb(0.16, 0.56, 0.42));
        } else if hovered.get() {
            *bg = BackgroundColor(Color::srgb(0.28, 0.35, 0.46));
        } else {
            *bg = BackgroundColor(Color::srgb(0.18, 0.23, 0.31));
        }
    }
}

fn update_tab_button_styles(
    ui_state: Res<WorkstationUiState>,
    mut btn_q: Query<(&TabButton, &Hovered, Option<&Pressed>, &mut BackgroundColor)>,
) {
    for (btn, hovered, pressed, mut bg) in &mut btn_q {
        if pressed.is_some() || ui_state.active_tab == btn.0 {
            *bg = BackgroundColor(Color::srgb(0.16, 0.56, 0.42));
        } else if hovered.get() {
            *bg = BackgroundColor(Color::srgb(0.28, 0.35, 0.46));
        } else {
            *bg = BackgroundColor(Color::srgb(0.18, 0.23, 0.31));
        }
    }
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
    mut overview_q: Query<
        (&mut Visibility, &mut Node),
        (
            With<OverviewBodyText>,
            Without<FinanceBodyText>,
            Without<WorkstationModal>,
            Without<CrosshairDot>,
        ),
    >,
    mut finance_q: Query<
        (&mut Visibility, &mut Node),
        (
            With<FinanceBodyText>,
            Without<OverviewBodyText>,
            Without<WorkstationModal>,
            Without<CrosshairDot>,
        ),
    >,
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
    if let Ok((mut ov_vis, mut ov_node)) = overview_q.single_mut() {
        let show = ui_state.active_tab == WorkstationTab::Overview;
        *ov_vis = if show { Visibility::Inherited } else { Visibility::Hidden };
        ov_node.display = if show { Display::Flex } else { Display::None };
    }
    if let Ok((mut fin_vis, mut fin_node)) = finance_q.single_mut() {
        let show = ui_state.active_tab == WorkstationTab::Finance;
        *fin_vis = if show { Visibility::Inherited } else { Visibility::Hidden };
        fin_node.display = if show { Display::Flex } else { Display::None };
    }
}

fn update_workstation_contents(
    ui_state: Res<WorkstationUiState>,
    company: Res<CompanyState>,
    clock: Res<GameClock>,
    finance: Res<FinanceState>,
    mut overview_q: Query<&mut Text, (With<OverviewBodyText>, Without<FinanceBodyText>)>,
    mut finance_q: Query<&mut Text, (With<FinanceBodyText>, Without<OverviewBodyText>)>,
) {
    if !ui_state.is_open {
        return;
    }
    if !ui_state.is_changed() && !company.is_changed() && !clock.is_changed() && !finance.is_changed() {
        return;
    }

    if let Ok(mut text) = overview_q.single_mut() {
        let warning = if finance.is_low_cash() {
            "\n[!] PERINGATAN: Kas perusahaan menipis di bawah $5,000!"
        } else {
            ""
        };
        text.0 = format!(
            "Ringkasan Perusahaan\n\
             • Nama         : {}\n\
             • Industri     : {}\n\
             • Kantor       : {}\n\
             • Hari & Waktu : Hari {} · {} ({})\n\
             • Kas Saat Ini : {}\n\
             • Laba/Rugi    : {}\n\
             • Reputasi     : {:.1}\n\
             • Tim          : {} orang (Founder){}",
            company.name,
            company.industry,
            company.office_tier,
            clock.day,
            clock.formatted_time(),
            clock.speed.label(),
            format_money(finance.cash_balance),
            format_money(finance.net_profit()),
            company.reputation,
            company.team_size,
            warning,
        );
    }

    if let Ok(mut text) = finance_q.single_mut() {
        let mut ledger = String::new();
        for tx in finance.transactions.iter().rev().take(8) {
            let sign = if tx.amount >= 0 { "+" } else { "" };
            ledger.push_str(&format!(
                "\n  [Hari {:>2} | {:<6}] {:<25} {}{}",
                tx.day,
                tx.kind.tag(),
                tx.label,
                sign,
                format_money(tx.amount)
            ));
        }

        text.0 = format!(
            "Laporan Keuangan\n\
             • Saldo Kas        : {}\n\
             • Total Pemasukan  : {}\n\
             • Total Pengeluaran: {}\n\
             • Laba / Rugi      : {}\n\
             • Sewa Kantor      : -{}/hari\n\n\
             Transaksi Terbaru:{}",
            format_money(finance.cash_balance),
            format_money(finance.total_revenue),
            format_money(finance.total_expenses),
            format_money(finance.net_profit()),
            format_money(finance.daily_rent),
            ledger
        );
    }
}
