use bevy::{
    app::AppExit,
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Activate, Button},
    window::{CursorGrabMode, CursorOptions},
};

use crate::{
    audio::AudioSettings,
    core::GameClock,
    economy::FinanceState,
    employee::{EmployeeCommand, EmployeeState},
    interaction::{InteractTriggered, InteractableKind},
    life::{
        CompanyStageState, DeliveryState, FoodItemKind, LifeCommand, PlayerStatus, QuestState,
        STAGE_2_UPGRADE_COST,
    },
    market::MarketState,
    office::OfficeExpansionState,
    player::{CameraMode, PlayerController},
    product::ProductState,
    save_system::SaveGameCommand,
    ui::{
        format_money, ActionButton, ActiveModal, HudInteractive, TabPanel, WorkstationTab,
        WorkstationUiState,
    },
};

#[derive(Component)]
pub struct HudPlayerSummaryText;

#[derive(Component)]
pub struct HudEnergyFill;

#[derive(Component)]
pub struct HudHungerFill;

#[derive(Component)]
pub struct HudXpFill;

#[derive(Component)]
pub struct HudEatQuickButton;

#[derive(Component)]
pub struct HudEatQuickText;

#[derive(Component)]
pub struct HudQuestTrackerText;

#[derive(Component)]
pub struct FounderTabBodyText;

#[derive(Component)]
pub struct Stage2UpgradeButton;

#[derive(Component)]
pub struct OrderTabBodyText;

#[derive(Component)]
pub struct OrderBagSummaryText;

#[derive(Component)]
pub struct MainMenuModal;

#[derive(Component)]
pub struct MainMenuSummaryText;

#[derive(Component)]
pub struct NpcInteractModal;

#[derive(Component)]
pub struct NpcInteractBodyText;

pub fn spawn_player_hud_and_quests(commands: &mut Commands) {
    // Top-left Player Status Card + Compact Quest Tracker
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(14.0),
                left: px(14.0),
                width: px(312.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(8.0),
                ..default()
            },
        ))
        .with_children(|col| {
            // Player Needs & Level Card
            col.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6.0),
                    padding: UiRect::all(px(11.0)),
                    border: UiRect::all(px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.11, 0.16, 0.86)),
                BorderColor::all(Color::srgb(0.22, 0.34, 0.46)),
            ))
            .with_children(|card| {
                card.spawn((
                    HudPlayerSummaryText,
                    Text::new("Stage 1 · Lv.1 | Semangat: Bersemangat"),
                    TextFont {
                        font_size: FontSize::Px(13.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.97, 1.0)),
                ));

                // XP bar
                spawn_stat_bar(
                    card,
                    "XP",
                    Color::srgb(0.35, 0.65, 0.96),
                    HudXpFill,
                    0.0,
                );
                // Energy bar
                spawn_stat_bar(
                    card,
                    "Energi",
                    Color::srgb(0.24, 0.84, 0.62),
                    HudEnergyFill,
                    92.0,
                );
                // Hunger bar
                spawn_stat_bar(
                    card,
                    "Kenyang",
                    Color::srgb(0.95, 0.72, 0.28),
                    HudHungerFill,
                    85.0,
                );

                // Quick eat button
                card.spawn((
                    HudEatQuickButton,
                    HudInteractive,
                    ActionButton,
                    Button,
                    Hovered::default(),
                    Node {
                        align_self: AlignSelf::FlexStart,
                        padding: UiRect::axes(px(10.0), px(5.0)),
                        margin: UiRect::top(px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.52, 0.40)),
                ))
                .observe(|_: On<Activate>, mut writer: MessageWriter<LifeCommand>| {
                    writer.write(LifeCommand::EatFromBag(0));
                })
                .with_children(|b| {
                    b.spawn((
                        HudEatQuickText,
                        Text::new("Makan Bekal (1)"),
                        TextFont {
                            font_size: FontSize::Px(12.5),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            });

            // Active Quest Card
            col.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4.0),
                    padding: UiRect::all(px(10.0)),
                    border: UiRect::all(px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.14, 0.78)),
                BorderColor::all(Color::srgb(0.22, 0.30, 0.42)),
            ))
            .with_children(|qbox| {
                qbox.spawn((
                    HudQuestTrackerText,
                    Text::new("Misi Aktif..."),
                    TextFont {
                        font_size: FontSize::Px(12.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.88, 0.93, 0.98)),
                ));
            });
        });

    // Bottom-Center Quick Pop-up Action Bar
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(14.0),
                width: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|wrap| {
            wrap.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(8.0),
                    padding: UiRect::axes(px(12.0), px(8.0)),
                    border: UiRect::all(px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.11, 0.16, 0.88)),
                BorderColor::all(Color::srgb(0.22, 0.34, 0.46)),
            ))
            .with_children(|bar| {
                for (label, target_tab) in [
                    ("Status & Misi", WorkstationTab::Founder),
                    ("Kelola Usaha", WorkstationTab::Overview),
                    ("Order / Kurir", WorkstationTab::Order),
                    ("Tata Kantor", WorkstationTab::Office),
                ] {
                    bar.spawn((
                        HudInteractive,
                        ActionButton,
                        Button,
                        Hovered::default(),
                        Node {
                            padding: UiRect::axes(px(12.0), px(7.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.16, 0.52, 0.40)),
                    ))
                    .observe(
                        move |_: On<Activate>,
                              mut ui_state: ResMut<WorkstationUiState>,
                              mut cursor_options: Single<&mut CursorOptions>| {
                            ui_state.open_workstation(target_tab);
                            cursor_options.visible = true;
                            cursor_options.grab_mode = CursorGrabMode::None;
                        },
                    )
                    .with_children(|b| {
                        b.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: FontSize::Px(13.5),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                }

                bar.spawn((
                    HudInteractive,
                    Button,
                    Hovered::default(),
                    Node {
                        padding: UiRect::axes(px(12.0), px(7.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.24, 0.29, 0.38)),
                ))
                .observe(
                    |_: On<Activate>,
                     mut ui_state: ResMut<WorkstationUiState>,
                     mut cursor_options: Single<&mut CursorOptions>| {
                        ui_state.open_main_menu();
                        cursor_options.visible = true;
                        cursor_options.grab_mode = CursorGrabMode::None;
                    },
                )
                .with_children(|b| {
                    b.spawn((
                        Text::new("Main Menu"),
                        TextFont {
                            font_size: FontSize::Px(13.5),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            });
        });
}

fn spawn_stat_bar<M: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    fill_color: Color,
    marker: M,
    initial_pct: f32,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.84, 0.92)),
                Node {
                    width: px(54.0),
                    ..default()
                },
            ));
            row.spawn((
                Node {
                    width: percent(100.0),
                    height: px(9.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.14, 0.18, 0.25)),
            ))
            .with_children(|track| {
                track.spawn((
                    marker,
                    Node {
                        width: percent(initial_pct),
                        height: percent(100.0),
                        ..default()
                    },
                    BackgroundColor(fill_color),
                ));
            });
        });
}

pub fn spawn_founder_and_order_tabs(panel: &mut ChildSpawnerCommands) {
    // Founder Status, Stage & Quests Tab
    panel
        .spawn((
            TabPanel(WorkstationTab::Founder),
            Visibility::Hidden,
            Node {
                display: Display::None,
                flex_direction: FlexDirection::Column,
                row_gap: px(10.0),
                padding: UiRect::all(px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.14, 0.18, 0.25)),
        ))
        .with_children(|fbox| {
            fbox.spawn((
                FounderTabBodyText,
                Text::new("Memuat status founder..."),
                TextFont {
                    font_size: FontSize::Px(13.5),
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.95, 0.98)),
            ));

            fbox.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(8.0),
                row_gap: px(8.0),
                ..default()
            })
            .with_children(|actions| {
                actions
                    .spawn((
                        ActionButton,
                        Button,
                        Hovered::default(),
                        Node {
                            padding: UiRect::axes(px(12.0), px(7.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.16, 0.52, 0.40)),
                    ))
                    .observe(|_: On<Activate>, mut writer: MessageWriter<LifeCommand>| {
                        writer.write(LifeCommand::EatFromBag(0));
                    })
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Makan Bekal dari Tas"),
                            TextFont {
                                font_size: FontSize::Px(13.5),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                actions
                    .spawn((
                        ActionButton,
                        Button,
                        Hovered::default(),
                        Node {
                            padding: UiRect::axes(px(12.0), px(7.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.16, 0.52, 0.40)),
                    ))
                    .observe(|_: On<Activate>, mut writer: MessageWriter<LifeCommand>| {
                        writer.write(LifeCommand::RestFounder);
                    })
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Istirahat Sebentar (+35 Energi)"),
                            TextFont {
                                font_size: FontSize::Px(13.5),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                actions
                    .spawn((
                        Stage2UpgradeButton,
                        ActionButton,
                        Button,
                        Hovered::default(),
                        Node {
                            padding: UiRect::axes(px(12.0), px(7.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.22, 0.48, 0.72)),
                    ))
                    .observe(|_: On<Activate>, mut writer: MessageWriter<LifeCommand>| {
                        writer.write(LifeCommand::UpgradeToStage2);
                    })
                    .with_children(|b| {
                        b.spawn((
                            Text::new(format!(
                                "Naik ke Stage 2: Small Startup ({})",
                                format_money(STAGE_2_UPGRADE_COST)
                            )),
                            TextFont {
                                font_size: FontSize::Px(13.5),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });
        });

    // Order & Courier Delivery Tab
    panel
        .spawn((
            TabPanel(WorkstationTab::Order),
            Visibility::Hidden,
            Node {
                display: Display::None,
                flex_direction: FlexDirection::Column,
                row_gap: px(10.0),
                padding: UiRect::all(px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.14, 0.18, 0.25)),
        ))
        .with_children(|obox| {
            obox.spawn((
                OrderTabBodyText,
                Text::new("Pesan Makanan & Perlengkapan Kantor (diantar kurir ke depan pintu)"),
                TextFont {
                    font_size: FontSize::Px(13.5),
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.95, 0.98)),
            ));

            obox.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(8.0),
                row_gap: px(8.0),
                ..default()
            })
            .with_children(|food_grid| {
                for item in FoodItemKind::ALL {
                    food_grid
                        .spawn((
                            ActionButton,
                            Button,
                            Hovered::default(),
                            Node {
                                width: px(220.0),
                                padding: UiRect::all(px(9.0)),
                                border: UiRect::all(px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.16, 0.52, 0.40)),
                            BorderColor::all(Color::srgb(0.24, 0.44, 0.50)),
                        ))
                        .observe(
                            move |_: On<Activate>, mut writer: MessageWriter<LifeCommand>| {
                                writer.write(LifeCommand::OrderFood(item));
                            },
                        )
                        .with_children(|b| {
                            b.spawn((
                                Text::new(format!(
                                    "{} ({})\n{}",
                                    item.label(),
                                    format_money(item.cost()),
                                    item.effect_summary()
                                )),
                                TextFont {
                                    font_size: FontSize::Px(12.5),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                }
            });

            obox.spawn((
                OrderBagSummaryText,
                Text::new("Bekal di tas & status kurir..."),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::srgb(0.84, 0.92, 0.96)),
            ));
        });
}

pub fn spawn_main_menu_and_npc_modals(commands: &mut Commands) {
    // Main Menu Pop-up Modal
    commands
        .spawn((
            MainMenuModal,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.05, 0.09, 0.72)),
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn((
                    Node {
                        width: px(380.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(10.0),
                        padding: UiRect::all(px(22.0)),
                        border: UiRect::all(px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.10, 0.13, 0.19)),
                    BorderColor::all(Color::srgb(0.20, 0.72, 0.54)),
                ))
                .with_children(|menu| {
                    menu.spawn((
                        Text::new("Main Menu - Startup Tycoon"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.24, 0.84, 0.62)),
                    ));
                    menu.spawn((
                        MainMenuSummaryText,
                        Text::new("Stage 1 · Hari 1"),
                        TextFont {
                            font_size: FontSize::Px(13.5),
                            ..default()
                        },
                        TextColor(Color::srgb(0.82, 0.88, 0.95)),
                    ));

                    spawn_menu_button(menu, "Lanjutkan Bermain").observe(
                        |_: On<Activate>, mut ui: ResMut<WorkstationUiState>| {
                            ui.close_all();
                        },
                    );
                    spawn_menu_button(menu, "Simpan Game (F5)").observe(
                        |_: On<Activate>, mut w: MessageWriter<SaveGameCommand>| {
                            w.write(SaveGameCommand::Save);
                        },
                    );
                    spawn_menu_button(menu, "Muat Save Terakhir (F9)").observe(
                        |_: On<Activate>,
                         mut w: MessageWriter<SaveGameCommand>,
                         mut ui: ResMut<WorkstationUiState>| {
                            w.write(SaveGameCommand::Load);
                            ui.close_all();
                        },
                    );
                    spawn_menu_button(menu, "Nyalakan / Matikan Suara (M)").observe(
                        |_: On<Activate>, mut audio: ResMut<AudioSettings>| {
                            audio.muted = !audio.muted;
                        },
                    );
                    spawn_menu_button(menu, "Ganti Kamera 1st / 3rd Person").observe(
                        |_: On<Activate>, mut q: Query<(&mut PlayerController, &mut Visibility)>| {
                            if let Ok((mut ctrl, mut vis)) = q.single_mut() {
                                ctrl.camera_mode = match ctrl.camera_mode {
                                    CameraMode::FirstPerson => {
                                        *vis = Visibility::Inherited;
                                        CameraMode::ThirdPerson
                                    }
                                    CameraMode::ThirdPerson => {
                                        *vis = Visibility::Hidden;
                                        CameraMode::FirstPerson
                                    }
                                };
                            }
                        },
                    );
                    spawn_menu_button(menu, "Keluar dari Game").observe(
                        |_: On<Activate>, mut exit: MessageWriter<AppExit>| {
                            exit.write(AppExit::Success);
                        },
                    );
                });
        });

    // NPC Employee Interaction Pop-up Modal
    commands
        .spawn((
            NpcInteractModal,
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
                        width: px(430.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(12.0),
                        padding: UiRect::all(px(20.0)),
                        border: UiRect::all(px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.10, 0.13, 0.19)),
                    BorderColor::all(Color::srgb(0.24, 0.72, 0.56)),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new("Obrolan & Kondisi Karyawan"),
                        TextFont {
                            font_size: FontSize::Px(19.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.24, 0.84, 0.62)),
                    ));
                    card.spawn((
                        NpcInteractBodyText,
                        Text::new("Memuat data karyawan..."),
                        TextFont {
                            font_size: FontSize::Px(14.5),
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.95, 0.98)),
                    ));

                    card.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(8.0),
                        row_gap: px(8.0),
                        ..default()
                    })
                    .with_children(|btns| {
                        spawn_menu_button(btns, "Beri Semangat").observe(
                            |_: On<Activate>,
                             ui: Res<WorkstationUiState>,
                             mut writer: MessageWriter<EmployeeCommand>| {
                                if let ActiveModal::NpcDialog(id) = ui.modal {
                                    writer.write(EmployeeCommand::CheerUp(id));
                                }
                            },
                        );
                        spawn_menu_button(btns, "Ubah Fokus Kerja").observe(
                            |_: On<Activate>,
                             ui: Res<WorkstationUiState>,
                             mut writer: MessageWriter<EmployeeCommand>| {
                                if let ActiveModal::NpcDialog(id) = ui.modal {
                                    writer.write(EmployeeCommand::ToggleFocus(id));
                                }
                            },
                        );
                        spawn_menu_button(btns, "Tutup [ESC / E]").observe(
                            |_: On<Activate>, mut ui: ResMut<WorkstationUiState>| {
                                ui.close_all();
                            },
                        );
                    });
                });
        });
}

fn spawn_menu_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    label: &str,
) -> EntityCommands<'a> {
    let mut entity = parent.spawn((
        ActionButton,
        Button,
        Hovered::default(),
        Node {
            padding: UiRect::axes(px(14.0), px(9.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.16, 0.52, 0.40)),
    ));
    entity.with_children(|b| {
        b.spawn((
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
    entity
}

pub fn handle_employee_npc_interact(
    mut reader: MessageReader<InteractTriggered>,
    emp_q: Query<&crate::employee::DeveloperNpc>,
    mut ui_state: ResMut<WorkstationUiState>,
    mut status: ResMut<PlayerStatus>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    for ev in reader.read() {
        if ev.kind == InteractableKind::Employee
            && let Ok(npc) = emp_q.get(ev.entity)
        {
            status.npc_chats_count += 1;
            ui_state.open_npc_dialog(npc.employee_id);
            cursor_options.visible = true;
            cursor_options.grab_mode = CursorGrabMode::None;
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_player_hud_and_quests(
    stage: Res<CompanyStageState>,
    status: Res<PlayerStatus>,
    product: Res<ProductState>,
    employees: Res<EmployeeState>,
    market: Res<MarketState>,
    expansion: Res<OfficeExpansionState>,
    quests: Res<QuestState>,
    mut summary_q: Query<
        &mut Text,
        (
            With<HudPlayerSummaryText>,
            Without<HudEatQuickText>,
            Without<HudQuestTrackerText>,
        ),
    >,
    mut eat_text_q: Query<
        &mut Text,
        (
            With<HudEatQuickText>,
            Without<HudPlayerSummaryText>,
            Without<HudQuestTrackerText>,
        ),
    >,
    mut quest_q: Query<
        &mut Text,
        (
            With<HudQuestTrackerText>,
            Without<HudPlayerSummaryText>,
            Without<HudEatQuickText>,
        ),
    >,
    mut bars_q: Query<
        (&mut Node, Option<&HudEnergyFill>, Option<&HudHungerFill>, Option<&HudXpFill>),
    >,
) {
    if !stage.is_changed()
        && !status.is_changed()
        && !product.is_changed()
        && !employees.is_changed()
        && !market.is_changed()
        && !expansion.is_changed()
    {
        return;
    }

    if let Ok(mut txt) = summary_q.single_mut() {
        txt.0 = format!(
            "{} · Lv.{} ({}/{}) | {}",
            stage.current.title(),
            status.level,
            status.xp,
            status.xp_to_next_level(),
            status.mood_label()
        );
    }

    if let Ok(mut eat_txt) = eat_text_q.single_mut() {
        if let Some(first) = status.snack_bag.first() {
            eat_txt.0 = format!("Makan {} ({})", first.label(), status.snack_bag.len());
        } else {
            eat_txt.0 = "Tas Bekal Kosong (Order)".to_string();
        }
    }

    let xp_pct =
        ((status.xp as f32 / status.xp_to_next_level() as f32) * 100.0).clamp(0.0, 100.0);
    for (mut node, e_fill, h_fill, x_fill) in &mut bars_q {
        if e_fill.is_some() {
            node.width = percent(status.energy.clamp(0.0, 100.0));
        } else if h_fill.is_some() {
            node.width = percent(status.hunger.clamp(0.0, 100.0));
        } else if x_fill.is_some() {
            node.width = percent(xp_pct);
        }
    }

    if let Ok(mut q_txt) = quest_q.single_mut() {
        let all = quests.build_quest_list(
            stage.current,
            &status,
            &product,
            &employees,
            &market,
            &expansion,
        );
        let mut lines = String::from("Misi Aktif:");
        let mut shown = 0;
        for q in &all {
            if !q.done && shown < 3 {
                lines.push_str(&format!("\n• {} (+{} XP)\n  {}", q.title, q.xp_reward, q.detail));
                shown += 1;
            }
        }
        if shown == 0 {
            lines.push_str("\n• Semua misi stage saat ini selesai!");
        }
        q_txt.0 = lines;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_founder_tab(
    ui_state: Res<WorkstationUiState>,
    stage: Res<CompanyStageState>,
    status: Res<PlayerStatus>,
    product: Res<ProductState>,
    employees: Res<EmployeeState>,
    market: Res<MarketState>,
    finance: Res<FinanceState>,
    expansion: Res<OfficeExpansionState>,
    quests: Res<QuestState>,
    mut body_q: Query<&mut Text, With<FounderTabBodyText>>,
    mut upgrade_q: Query<(&mut Visibility, &mut Node), With<Stage2UpgradeButton>>,
) {
    if !ui_state.is_open || ui_state.active_tab != WorkstationTab::Founder {
        return;
    }

    if let Ok(mut txt) = body_q.single_mut() {
        let quest_items = quests.build_quest_list(
            stage.current,
            &status,
            &product,
            &employees,
            &market,
            &expansion,
        );
        let mut quest_str = String::new();
        for q in quest_items {
            let mark = if q.done { "[SELESAI]" } else { "[AKTIF]" };
            quest_str.push_str(&format!("\n  {mark} {} (+{} XP) — {}", q.title, q.xp_reward, q.detail));
        }

        let stage_status = match stage.can_upgrade_to_stage_2(&status, &product, &market, &finance) {
            Ok(()) => "SIAP NAIK KE STAGE 2: SMALL STARTUP!".to_string(),
            Err(msg) => format!("Syarat Stage 2: {msg}"),
        };

        txt.0 = format!(
            "Status Founder & Perjalanan Usaha\n\
             • Stage Saat Ini : {} ({})\n\
             • Level Founder  : Level {} ({} / {} XP)\n\
             • Kondisi Tubuh  : Energi {:.0}% | Kenyang {:.0}% | Semangat {:.0}% ({})\n\
             • Kecepatan Kerja: {:.0}% efisiensi\n\
             • Tas Bekal      : {} item siap dimakan\n\
             • Status Stage   : {stage_status}\n\n\
             Roadmap Hidup: [1] Founder -> [2] Small Startup -> [3] Growing Company -> [4] Corporation\n\n\
             Daftar Misi:{quest_str}",
            stage.current.title(),
            stage.current.place_label(),
            status.level,
            status.xp,
            status.xp_to_next_level(),
            status.energy,
            status.hunger,
            status.mood,
            status.mood_label(),
            status.work_efficiency() * 100.0,
            status.snack_bag.len(),
        );
    }

    if let Ok((mut vis, mut node)) = upgrade_q.single_mut() {
        let show = stage.current == crate::life::CompanyStage::Founder;
        *vis = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        node.display = if show { Display::Flex } else { Display::None };
    }
}

pub fn update_order_tab(
    ui_state: Res<WorkstationUiState>,
    status: Res<PlayerStatus>,
    delivery: Res<DeliveryState>,
    mut bag_q: Query<&mut Text, With<OrderBagSummaryText>>,
) {
    if !ui_state.is_open || ui_state.active_tab != WorkstationTab::Order {
        return;
    }

    if let Ok(mut txt) = bag_q.single_mut() {
        let bag_list = if status.snack_bag.is_empty() {
            "Kosong".to_string()
        } else {
            status
                .snack_bag
                .iter()
                .map(|f| f.label())
                .collect::<Vec<_>>()
                .join(", ")
        };

        let mut order_lines = String::new();
        for o in &delivery.orders {
            let st = if o.arrived {
                "TIBA DI DEPAN PINTU — Dekati pintu & tekan [E]!"
            } else {
                "Sedang diantar kurir..."
            };
            order_lines.push_str(&format!("\n  • {} — {st}", o.payload.label()));
        }
        if order_lines.is_empty() {
            order_lines.push_str("\n  Tidak ada pengiriman aktif.");
        }

        txt.0 = format!(
            "Tas Bekal Founder ({} item): {bag_list}\n\
             Status Pengiriman Kurir:{order_lines}\n\
             Tip: Perlengkapan kantor di tab 'Kantor' juga diantar kurir ke depan pintu!",
            status.snack_bag.len()
        );
    }
}

pub fn sync_extra_modals_and_content(
    ui_state: Res<WorkstationUiState>,
    stage: Res<CompanyStageState>,
    clock: Res<GameClock>,
    employees: Res<EmployeeState>,
    mut menu_q: Query<&mut Visibility, (With<MainMenuModal>, Without<NpcInteractModal>)>,
    mut npc_q: Query<&mut Visibility, (With<NpcInteractModal>, Without<MainMenuModal>)>,
    mut menu_txt_q: Query<&mut Text, (With<MainMenuSummaryText>, Without<NpcInteractBodyText>)>,
    mut npc_txt_q: Query<&mut Text, (With<NpcInteractBodyText>, Without<MainMenuSummaryText>)>,
) {
    if let Ok(mut vis) = menu_q.single_mut() {
        *vis = if ui_state.modal == ActiveModal::MainMenu {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    let npc_target = match ui_state.modal {
        ActiveModal::NpcDialog(id) => Some(id),
        _ => None,
    };

    if let Ok(mut vis) = npc_q.single_mut() {
        *vis = if npc_target.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if ui_state.modal == ActiveModal::MainMenu
        && let Ok(mut txt) = menu_txt_q.single_mut()
    {
        txt.0 = format!(
            "{} · Hari {} ({})",
            stage.current.title(),
            clock.day,
            clock.formatted_time()
        );
    }

    if let Some(emp_id) = npc_target
        && let Ok(mut txt) = npc_txt_q.single_mut()
    {
        if let Some(emp) = employees.roster.iter().find(|e| e.id == emp_id) {
            txt.0 = format!(
                "Nama       : {}\n\
                 Peran      : {}\n\
                 Aktivitas  : {}\n\
                 Fokus Kerja: {}\n\
                 Energi     : {:.0}% | Semangat: {:.0}%\n\
                 Gaji Harian: {}/hari",
                emp.name,
                emp.role,
                emp.state.label(),
                emp.focus.label(),
                emp.energy,
                emp.mood,
                format_money(emp.daily_salary),
            );
        }
    }
}
