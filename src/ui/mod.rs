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
    employee::{EmployeeCommand, EmployeeState},
    interaction::{ActiveInteractionTarget, InteractTriggered, InteractableKind},
    market::{MarketCommand, MarketState},
    player::{CameraMode, PlayerController},
    product::{ProductCommand, ProductPhase, ProductState, PRODUCT_DAILY_DEV_COST, PRODUCT_START_COST},
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorkstationUiState>()
            .add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    (handle_workstation_close, handle_workstation_interact).chain(),
                    update_hud_mode,
                    update_hud_status,
                    update_speed_button_styles,
                    update_tab_button_styles,
                    update_price_button_styles,
                    update_action_button_styles,
                    update_interaction_prompt,
                    sync_workstation_modal_visibility,
                    update_overview_tab,
                    update_product_tab,
                    update_employee_tab,
                    update_market_tab,
                    update_finance_tab,
                ),
            );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkstationTab {
    #[default]
    Overview,
    Product,
    Employee,
    Market,
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

#[derive(Component, Clone, Copy)]
struct TabPanel(WorkstationTab);

#[derive(Component)]
struct OverviewBodyText;

#[derive(Component)]
struct ProductBodyText;

#[derive(Component)]
struct ProductStarterGroup;

#[derive(Component)]
struct ProductProgressBarFill;

#[derive(Component)]
struct ProductLaunchButton;

#[derive(Component)]
struct ActionButton;

#[derive(Component)]
struct EmployeeBodyText;

#[derive(Component)]
struct EmployeeHireGroup;

#[derive(Component)]
struct EmployeeRosterPortrait;

#[derive(Component)]
struct MarketBodyText;

#[derive(Component, Clone, Copy)]
struct PriceButton(i64);

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

fn hud_hint_string(mode_label: &str) -> String {
    format!(
        "Startup Tycoon (M6) - Mode: {mode_label}\n\
         [Klik Kanan] Kunci Mouse | [ESC] Lepas Mouse\n\
         [WASD] Jalan | [Shift] Lari | [V] 1st/3rd Person | [E] Interaksi\n\
         [0/P] Pause | [1] 1x | [2] 2x"
    )
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon_money: Handle<Image> = asset_server.load("ui/icons/ui_icon_money.png");
    let icon_time: Handle<Image> = asset_server.load("ui/icons/ui_icon_time.png");
    let icon_company: Handle<Image> = asset_server.load("ui/icons/ui_icon_company.png");

    // Top-left controls & mode overlay
    commands.spawn((
        HudHintText,
        Text::new(hud_hint_string("3rd Person")),
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
                    Text::new("Hari 1 · 09:00 [1x]"),
                    TextFont {
                        font_size: FontSize::Px(15.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.95, 1.0)),
                ));
            });

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
                        width: px(720.0),
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
                                        Text::new("StartupOS v0.3 - Manajemen Perusahaan"),
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
                                (WorkstationTab::Product, "Product"),
                                (WorkstationTab::Employee, "Employee"),
                                (WorkstationTab::Market, "Market"),
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
                        TabPanel(WorkstationTab::Overview),
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

                    // Product Tab Container
                    panel
                        .spawn((
                            TabPanel(WorkstationTab::Product),
                            Visibility::Hidden,
                            Node {
                                display: Display::None,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(12.0),
                                padding: UiRect::all(px(14.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.14, 0.18, 0.25)),
                        ))
                        .with_children(|prod_box| {
                            prod_box.spawn((
                                ProductBodyText,
                                Text::new("Belum ada proyek produk aktif."),
                                TextFont {
                                    font_size: FontSize::Px(15.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.92, 0.94, 0.98)),
                            ));

                            // Progress bar track
                            prod_box
                                .spawn((
                                    Node {
                                        width: percent(100.0),
                                        height: px(14.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.08, 0.11, 0.16)),
                                ))
                                .with_children(|track| {
                                    track.spawn((
                                        ProductProgressBarFill,
                                        Node {
                                            width: percent(0.0),
                                            height: percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.20, 0.78, 0.56)),
                                    ));
                                });

                            // Starter presets (shown when no active product)
                            prod_box
                                .spawn((
                                    ProductStarterGroup,
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        flex_wrap: FlexWrap::Wrap,
                                        column_gap: px(8.0),
                                        row_gap: px(8.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|starters| {
                                    for (name, cat) in [
                                        ("TaskFlow SaaS", "Productivity"),
                                        ("CloudNotes App", "Collaboration"),
                                        ("BugTracker Pro", "DevTools"),
                                    ] {
                                        let name_owned = name.to_string();
                                        let cat_owned = cat.to_string();
                                        starters
                                            .spawn((
                                                ActionButton,
                                                Button,
                                                Hovered::default(),
                                                Node {
                                                    padding: UiRect::axes(px(12.0), px(8.0)),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgb(0.16, 0.52, 0.40)),
                                            ))
                                            .observe(
                                                move |_: On<Activate>,
                                                      mut writer: MessageWriter<ProductCommand>| {
                                                    writer.write(ProductCommand::StartProject {
                                                        name: name_owned.clone(),
                                                        category: cat_owned.clone(),
                                                    });
                                                },
                                            )
                                            .with_children(|b| {
                                                b.spawn((
                                                    Text::new(format!(
                                                        "Buat {name} ({})",
                                                        format_money(PRODUCT_START_COST)
                                                    )),
                                                    TextFont {
                                                        font_size: FontSize::Px(14.0),
                                                        ..default()
                                                    },
                                                    TextColor(Color::WHITE),
                                                ));
                                            });
                                    }
                                });

                            // Launch button (shown only when ReadyToLaunch)
                            prod_box
                                .spawn((
                                    ProductLaunchButton,
                                    ActionButton,
                                    Visibility::Hidden,
                                    Button,
                                    Hovered::default(),
                                    Node {
                                        display: Display::None,
                                        align_self: AlignSelf::FlexStart,
                                        padding: UiRect::axes(px(16.0), px(9.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.16, 0.56, 0.42)),
                                ))
                                .observe(
                                    |_: On<Activate>, mut writer: MessageWriter<ProductCommand>| {
                                        writer.write(ProductCommand::LaunchProduct);
                                    },
                                )
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new("Rilis Produk Sekarang!"),
                                        TextFont {
                                            font_size: FontSize::Px(15.0),
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });

                    // Employee Tab Container
                    panel
                        .spawn((
                            TabPanel(WorkstationTab::Employee),
                            Visibility::Hidden,
                            Node {
                                display: Display::None,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(12.0),
                                padding: UiRect::all(px(14.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.14, 0.18, 0.25)),
                        ))
                        .with_children(|emp_box| {
                            emp_box
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: px(12.0),
                                    ..default()
                                })
                                .with_children(|row| {
                                    row.spawn((
                                        EmployeeRosterPortrait,
                                        Visibility::Hidden,
                                        ImageNode::new(
                                            asset_server.load(
                                                "characters/employees/portraits/developer_01.png",
                                            ),
                                        ),
                                        Node {
                                            display: Display::None,
                                            width: px(64.0),
                                            height: px(64.0),
                                            ..default()
                                        },
                                    ));
                                    row.spawn((
                                        EmployeeBodyText,
                                        Text::new("Belum ada karyawan."),
                                        TextFont {
                                            font_size: FontSize::Px(15.0),
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.92, 0.94, 0.98)),
                                    ));
                                });

                            emp_box
                                .spawn((
                                    EmployeeHireGroup,
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        column_gap: px(12.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|cands| {
                                    for (idx, name, skill, salary, fee, portrait) in [
                                        (
                                            0usize,
                                            "Raka Pratama",
                                            1.2f32,
                                            160i64,
                                            600i64,
                                            "characters/employees/portraits/developer_01.png",
                                        ),
                                        (
                                            1usize,
                                            "Sinta Wijaya",
                                            1.8f32,
                                            240i64,
                                            1_000i64,
                                            "characters/employees/portraits/developer_02.png",
                                        ),
                                    ] {
                                        cands
                                            .spawn((
                                                Node {
                                                    flex_direction: FlexDirection::Column,
                                                    row_gap: px(8.0),
                                                    padding: UiRect::all(px(12.0)),
                                                    border: UiRect::all(px(1.0)),
                                                    width: px(310.0),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgb(0.10, 0.13, 0.19)),
                                                BorderColor::all(Color::srgb(0.24, 0.34, 0.46)),
                                            ))
                                            .with_children(|card| {
                                                card.spawn(Node {
                                                    flex_direction: FlexDirection::Row,
                                                    align_items: AlignItems::Center,
                                                    column_gap: px(10.0),
                                                    ..default()
                                                })
                                                .with_children(|top| {
                                                    top.spawn((
                                                        ImageNode::new(asset_server.load(portrait)),
                                                        Node {
                                                            width: px(54.0),
                                                            height: px(54.0),
                                                            ..default()
                                                        },
                                                    ));
                                                    top.spawn((
                                                        Text::new(format!(
                                                            "{name}\nRole: Developer\nSkill: {skill:.1}x | Gaji: {}/hr",
                                                            format_money(salary)
                                                        )),
                                                        TextFont {
                                                            font_size: FontSize::Px(13.5),
                                                            ..default()
                                                        },
                                                        TextColor(Color::srgb(0.92, 0.95, 0.98)),
                                                    ));
                                                });

                                                card.spawn((
                                                    ActionButton,
                                                    Button,
                                                    Hovered::default(),
                                                    Node {
                                                        align_self: AlignSelf::FlexStart,
                                                        padding: UiRect::axes(px(12.0), px(7.0)),
                                                        ..default()
                                                    },
                                                    BackgroundColor(Color::srgb(0.16, 0.52, 0.40)),
                                                ))
                                                .observe(
                                                    move |_: On<Activate>,
                                                          mut writer: MessageWriter<EmployeeCommand>| {
                                                        writer.write(EmployeeCommand::HireCandidate(idx));
                                                    },
                                                )
                                                .with_children(|b| {
                                                    b.spawn((
                                                        Text::new(format!(
                                                            "Rekrut ({})",
                                                            format_money(fee)
                                                        )),
                                                        TextFont {
                                                            font_size: FontSize::Px(13.5),
                                                            ..default()
                                                        },
                                                        TextColor(Color::WHITE),
                                                    ));
                                                });
                                            });
                                    }
                                });
                        });

                    // Market Tab Container
                    panel
                        .spawn((
                            TabPanel(WorkstationTab::Market),
                            Visibility::Hidden,
                            Node {
                                display: Display::None,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(12.0),
                                padding: UiRect::all(px(14.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.14, 0.18, 0.25)),
                        ))
                        .with_children(|mkt_box| {
                            mkt_box.spawn((
                                MarketBodyText,
                                Text::new("Loading market report..."),
                                TextFont {
                                    font_size: FontSize::Px(14.5),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.92, 0.94, 0.98)),
                            ));

                            mkt_box
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(8.0),
                                    ..default()
                                })
                                .with_children(|pricing| {
                                    for (price, label) in [
                                        (10, "$10/bln (Ekonomis)"),
                                        (15, "$15/bln (Standar)"),
                                        (25, "$25/bln (Premium)"),
                                    ] {
                                        pricing
                                            .spawn((
                                                PriceButton(price),
                                                Button,
                                                Hovered::default(),
                                                Node {
                                                    padding: UiRect::axes(px(12.0), px(7.0)),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgb(0.18, 0.23, 0.31)),
                                            ))
                                            .observe(
                                                move |_: On<Activate>,
                                                      mut writer: MessageWriter<MarketCommand>| {
                                                    writer.write(MarketCommand::SetPrice(price));
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
                                });
                        });

                    // Finance Body
                    panel.spawn((
                        TabPanel(WorkstationTab::Finance),
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
                            "Tips: Bangun produk di tab Product, rekrut Developer di tab Employee, rilis, lalu atur harga & pantau omzet di tab Market.",
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
    text.0 = hud_hint_string(mode_label);
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

fn update_price_button_styles(
    market: Res<MarketState>,
    mut btn_q: Query<(&PriceButton, &Hovered, Option<&Pressed>, &mut BackgroundColor)>,
) {
    for (btn, hovered, pressed, mut bg) in &mut btn_q {
        if pressed.is_some() || market.price_per_month == btn.0 {
            *bg = BackgroundColor(Color::srgb(0.16, 0.56, 0.42));
        } else if hovered.get() {
            *bg = BackgroundColor(Color::srgb(0.28, 0.35, 0.46));
        } else {
            *bg = BackgroundColor(Color::srgb(0.18, 0.23, 0.31));
        }
    }
}

fn update_action_button_styles(
    mut btn_q: Query<(&Hovered, Option<&Pressed>, &mut BackgroundColor), With<ActionButton>>,
) {
    for (hovered, pressed, mut bg) in &mut btn_q {
        if pressed.is_some() {
            *bg = BackgroundColor(Color::srgb(0.12, 0.44, 0.33));
        } else if hovered.get() {
            *bg = BackgroundColor(Color::srgb(0.22, 0.68, 0.52));
        } else {
            *bg = BackgroundColor(Color::srgb(0.16, 0.52, 0.40));
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
    mut modal_q: Query<&mut Visibility, (With<WorkstationModal>, Without<CrosshairDot>, Without<TabPanel>)>,
    mut crosshair_q: Query<&mut Visibility, (With<CrosshairDot>, Without<WorkstationModal>, Without<TabPanel>)>,
    mut panels_q: Query<(&TabPanel, &mut Visibility, &mut Node), (Without<WorkstationModal>, Without<CrosshairDot>)>,
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

    for (panel, mut vis, mut node) in &mut panels_q {
        let show = ui_state.active_tab == panel.0;
        *vis = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        node.display = if show { Display::Flex } else { Display::None };
    }
}

fn update_overview_tab(
    ui_state: Res<WorkstationUiState>,
    company: Res<CompanyState>,
    clock: Res<GameClock>,
    finance: Res<FinanceState>,
    product: Res<ProductState>,
    market: Res<MarketState>,
    mut overview_q: Query<&mut Text, With<OverviewBodyText>>,
) {
    if !ui_state.is_open {
        return;
    }
    if !ui_state.is_changed()
        && !company.is_changed()
        && !clock.is_changed()
        && !finance.is_changed()
        && !product.is_changed()
        && !market.is_changed()
    {
        return;
    }

    let product_summary = match &product.active {
        None => "Belum ada proyek (buka tab Product)".to_string(),
        Some(p) => format!("{} ({} · {:.0}%)", p.name, p.phase.label(), p.progress),
    };

    if let Ok(mut text) = overview_q.single_mut() {
        let warning = if finance.is_low_cash() {
            "\n[!] PERINGATAN: Kas perusahaan menipis di bawah $5,000!"
        } else {
            ""
        };
        text.0 = format!(
            "Ringkasan Perusahaan\n\
             • Nama           : {}\n\
             • Industri       : {}\n\
             • Kantor         : {}\n\
             • Hari & Waktu   : Hari {} · {} ({})\n\
             • Kas Saat Ini   : {}\n\
             • Laba/Rugi      : {}\n\
             • Reputasi       : {:.1}\n\
             • Produk Aktif   : {}\n\
             • Pengguna Aktif : {} user (Omzet: {}/hari)\n\
             • Tim            : {} orang{}",
            company.name,
            company.industry,
            company.office_tier,
            clock.day,
            clock.formatted_time(),
            clock.speed.label(),
            format_money(finance.cash_balance),
            format_money(finance.net_profit()),
            company.reputation,
            product_summary,
            market.active_users,
            format_money(market.last_daily_revenue),
            company.team_size,
            warning,
        );
    }
}

fn update_product_tab(
    ui_state: Res<WorkstationUiState>,
    product: Res<ProductState>,
    mut product_q: Query<&mut Text, With<ProductBodyText>>,
    mut starter_q: Query<(&mut Visibility, &mut Node), (With<ProductStarterGroup>, Without<ProductLaunchButton>, Without<ProductProgressBarFill>)>,
    mut launch_q: Query<(&mut Visibility, &mut Node), (With<ProductLaunchButton>, Without<ProductStarterGroup>, Without<ProductProgressBarFill>)>,
    mut fill_q: Query<&mut Node, (With<ProductProgressBarFill>, Without<ProductStarterGroup>, Without<ProductLaunchButton>)>,
) {
    if !ui_state.is_open || (!ui_state.is_changed() && !product.is_changed()) {
        return;
    }

    if let Ok(mut text) = product_q.single_mut() {
        if let Some(p) = &product.active {
            let launch_info = match p.launch_day {
                Some(d) => format!("Hari {d}"),
                None => "Belum dirilis".to_string(),
            };
            text.0 = format!(
                "Pengembangan Produk Software\n\
                 • Nama Produk  : {}\n\
                 • Kategori     : {}\n\
                 • Fase         : {}\n\
                 • Progres Fase : {:.0}%\n\
                 • Kualitas     : {:.1} / 100\n\
                 • Mulai / Rilis: Hari {} / {}\n\
                 • Biaya R&D    : -{}/hari (saat Dev/Testing)",
                p.name,
                p.category,
                p.phase.label(),
                p.progress,
                p.quality,
                p.start_day,
                launch_info,
                format_money(PRODUCT_DAILY_DEV_COST),
            );
        } else {
            text.0 = format!(
                "Belum Ada Produk Aktif\n\
                 Pilih salah satu konsep produk software di bawah untuk mulai membangun (biaya inisiasi {}):",
                format_money(PRODUCT_START_COST)
            );
        }
    }

    if let Ok(mut fill_node) = fill_q.single_mut() {
        let pct = product.active.as_ref().map_or(0.0, |p| p.progress.clamp(0.0, 100.0));
        fill_node.width = percent(pct);
    }

    if let Ok((mut s_vis, mut s_node)) = starter_q.single_mut() {
        let show = product.active.is_none();
        *s_vis = if show { Visibility::Inherited } else { Visibility::Hidden };
        s_node.display = if show { Display::Flex } else { Display::None };
    }

    if let Ok((mut l_vis, mut l_node)) = launch_q.single_mut() {
        let show = product
            .active
            .as_ref()
            .is_some_and(|p| p.phase == ProductPhase::ReadyToLaunch);
        *l_vis = if show { Visibility::Inherited } else { Visibility::Hidden };
        l_node.display = if show { Display::Flex } else { Display::None };
    }
}

fn update_employee_tab(
    ui_state: Res<WorkstationUiState>,
    asset_server: Res<AssetServer>,
    employees: Res<EmployeeState>,
    mut employee_q: Query<&mut Text, With<EmployeeBodyText>>,
    mut hire_q: Query<(&mut Visibility, &mut Node), (With<EmployeeHireGroup>, Without<EmployeeRosterPortrait>)>,
    mut portrait_q: Query<(&mut Visibility, &mut Node, &mut ImageNode), (With<EmployeeRosterPortrait>, Without<EmployeeHireGroup>)>,
) {
    if !ui_state.is_open || (!ui_state.is_changed() && !employees.is_changed()) {
        return;
    }

    if let Ok(mut text) = employee_q.single_mut() {
        if let Some(emp) = employees.roster.first() {
            text.0 = format!(
                "Karyawan Aktif (1 / {} Workstation)\n\
                 • Nama     : {}\n\
                 • Role     : {}\n\
                 • Skill    : {:.1}x (Bonus dev +{:.1}%/jam)\n\
                 • Gaji     : -{}/hari\n\
                 • Status   : {}",
                employees.max_employees,
                emp.name,
                emp.role,
                emp.skill,
                emp.skill * 1.8,
                format_money(emp.daily_salary),
                emp.state.label(),
            );
        } else {
            text.0 = "Belum ada karyawan. Pilih kandidat Developer untuk mempercepat pengembangan produk:".to_string();
        }
    }

    if let Ok((mut h_vis, mut h_node)) = hire_q.single_mut() {
        let show = employees.roster.len() < employees.max_employees;
        *h_vis = if show { Visibility::Inherited } else { Visibility::Hidden };
        h_node.display = if show { Display::Flex } else { Display::None };
    }

    if let Ok((mut p_vis, mut p_node, mut p_img)) = portrait_q.single_mut() {
        if let Some(emp) = employees.roster.first() {
            *p_vis = Visibility::Inherited;
            p_node.display = Display::Flex;
            p_img.image = asset_server.load(emp.portrait_path);
        } else {
            *p_vis = Visibility::Hidden;
            p_node.display = Display::None;
        }
    }
}

fn update_market_tab(
    ui_state: Res<WorkstationUiState>,
    product: Res<ProductState>,
    market: Res<MarketState>,
    mut market_q: Query<&mut Text, With<MarketBodyText>>,
) {
    if !ui_state.is_open || (!ui_state.is_changed() && !product.is_changed() && !market.is_changed()) {
        return;
    }

    if let Ok(mut text) = market_q.single_mut() {
        let is_launched = product
            .active
            .as_ref()
            .is_some_and(|p| p.phase == ProductPhase::Launched);

        let mut hist_lines = String::new();
        for snap in market.history.iter().rev().take(6) {
            hist_lines.push_str(&format!(
                "\n  [Hari {:>2}] +{} baru / -{} churn | {} aktif | +{}",
                snap.day,
                snap.new_users,
                snap.churned_users,
                snap.active_users,
                format_money(snap.revenue),
            ));
        }
        if hist_lines.is_empty() {
            hist_lines.push_str(if is_launched {
                "\n  Menunggu pergantian hari pertama pasca-rilis..."
            } else {
                "\n  Rilis produk terlebih dahulu di tab Product untuk mulai mendapat user."
            });
        }

        text.0 = format!(
            "Analisis Pasar & Penjualan\n\
             • Kondisi Pasar    : {} ({:.2}x demand)\n\
             • Harga Langganan  : {}/user/bln (Churn {:.1}%/hari)\n\
             • Pengguna Aktif   : {} user\n\
             • Omzet Hari Lalu  : {}\n\n\
             Riwayat Harian:{}\n\n\
             Pilih Harga Langganan (murah = pertumbuhan cepat, mahal = margin tinggi):",
            market.demand_label(),
            market.base_demand,
            format_money(market.price_per_month),
            market.churn_rate() * 100.0,
            market.active_users,
            format_money(market.last_daily_revenue),
            hist_lines,
        );
    }
}

fn update_finance_tab(
    ui_state: Res<WorkstationUiState>,
    finance: Res<FinanceState>,
    mut finance_q: Query<&mut Text, With<FinanceBodyText>>,
) {
    if !ui_state.is_open || (!ui_state.is_changed() && !finance.is_changed()) {
        return;
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
