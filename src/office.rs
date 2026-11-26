use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_PI_2;

use crate::{
    company::CompanyState,
    core::{GameClock, NotifyMessage},
    economy::{FinanceState, TransactionKind},
    employee::EmployeeState,
    interaction::{InteractTriggered, Interactable, InteractableKind},
};

pub struct OfficePlugin;

impl Plugin for OfficePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OfficeColliders>()
            .init_resource::<OfficeExpansionState>()
            .add_message::<OfficeUpgradeCommand>()
            .add_systems(Startup, setup_office_scene)
            .add_systems(
                Update,
                (
                    handle_door_interactions,
                    animate_door,
                    handle_office_upgrade_commands,
                ),
            );
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
    pub room_min_x: f32,
    pub room_max_x: f32,
    pub room_half_z: f32,
}

#[derive(Component)]
pub struct SunLight;

#[derive(Component)]
pub struct WarmLampLight;

#[derive(Component)]
pub struct EastArchwayTile;

#[derive(Component)]
pub struct OfficeDoor {
    pub is_open: bool,
    pub current_angle: f32,
    pub target_angle: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OfficeUpgradeKind {
    OfficePlant,
    Whiteboard,
    LoungeSofa,
    ServerRack,
    ExtraWorkstation,
}

impl OfficeUpgradeKind {
    pub const ALL: [OfficeUpgradeKind; 5] = [
        OfficeUpgradeKind::OfficePlant,
        OfficeUpgradeKind::Whiteboard,
        OfficeUpgradeKind::LoungeSofa,
        OfficeUpgradeKind::ServerRack,
        OfficeUpgradeKind::ExtraWorkstation,
    ];

    pub fn label(self) -> &'static str {
        match self {
            OfficeUpgradeKind::OfficePlant => "Tanaman Hias",
            OfficeUpgradeKind::Whiteboard => "Papan Ide & Jadwal",
            OfficeUpgradeKind::LoungeSofa => "Sofa Istirahat",
            OfficeUpgradeKind::ServerRack => "Perangkat Layanan Cepat",
            OfficeUpgradeKind::ExtraWorkstation => "Meja Kerja Tim #2",
        }
    }

    pub fn effect_summary(self) -> &'static str {
        match self {
            OfficeUpgradeKind::OfficePlant => "+1.0 Reputasi, suasana makin segar",
            OfficeUpgradeKind::Whiteboard => "+1.5 Reputasi, kerja lebih terarah",
            OfficeUpgradeKind::LoungeSofa => "+2.0 Reputasi, bisa istirahat [E]",
            OfficeUpgradeKind::ServerRack => "+3.0 Reputasi, +12% minat pelanggan",
            OfficeUpgradeKind::ExtraWorkstation => "+1.5 Reputasi, tambah 1 slot tim",
        }
    }

    pub fn cost(self) -> i64 {
        match self {
            OfficeUpgradeKind::OfficePlant => 150,
            OfficeUpgradeKind::Whiteboard => 300,
            OfficeUpgradeKind::LoungeSofa => 450,
            OfficeUpgradeKind::ServerRack => 900,
            OfficeUpgradeKind::ExtraWorkstation => 1_200,
        }
    }

    pub fn rep_bonus(self) -> f32 {
        match self {
            OfficeUpgradeKind::OfficePlant => 1.0,
            OfficeUpgradeKind::Whiteboard => 1.5,
            OfficeUpgradeKind::LoungeSofa => 2.0,
            OfficeUpgradeKind::ServerRack => 3.0,
            OfficeUpgradeKind::ExtraWorkstation => 1.5,
        }
    }

    pub fn collider(self) -> Aabb2d {
        match self {
            OfficeUpgradeKind::OfficePlant => Aabb2d {
                min_x: 2.30,
                max_x: 2.80,
                min_z: -2.30,
                max_z: -1.80,
            },
            OfficeUpgradeKind::Whiteboard => Aabb2d {
                min_x: -0.60,
                max_x: 0.60,
                min_z: -2.40,
                max_z: -1.92,
            },
            OfficeUpgradeKind::LoungeSofa => Aabb2d {
                min_x: 1.65,
                max_x: 2.50,
                min_z: 0.72,
                max_z: 2.28,
            },
            OfficeUpgradeKind::ServerRack => Aabb2d {
                min_x: -2.80,
                max_x: -2.10,
                min_z: 1.65,
                max_z: 2.35,
            },
            OfficeUpgradeKind::ExtraWorkstation => Aabb2d {
                min_x: -2.60,
                max_x: -1.15,
                min_z: -0.15,
                max_z: 1.35,
            },
        }
    }
}

#[derive(Resource, Clone, Default, Debug, Serialize, Deserialize)]
pub struct OfficeExpansionState {
    pub purchased: Vec<OfficeUpgradeKind>,
}

impl OfficeExpansionState {
    pub fn is_purchased(&self, kind: OfficeUpgradeKind) -> bool {
        self.purchased.contains(&kind)
    }

    pub fn has_whiteboard(&self) -> bool {
        self.is_purchased(OfficeUpgradeKind::Whiteboard)
    }

    pub fn has_server_rack(&self) -> bool {
        self.is_purchased(OfficeUpgradeKind::ServerRack)
    }

    pub fn buy_upgrade(
        &mut self,
        kind: OfficeUpgradeKind,
        day: u32,
        finance: &mut FinanceState,
        company: &mut CompanyState,
        employees: &mut EmployeeState,
    ) -> Result<(), &'static str> {
        if self.is_purchased(kind) {
            return Err("Upgrade kantor ini sudah dimiliki.");
        }
        let cost = kind.cost();
        if finance.cash_balance < cost {
            return Err("Saldo kas tidak cukup untuk membeli aset kantor ini.");
        }

        finance.record_transaction(
            day,
            format!("Beli {}", kind.label()),
            -cost,
            TransactionKind::OfficeUpgrade,
        );
        self.purchased.push(kind);
        company.reputation = (company.reputation + kind.rep_bonus()).min(100.0);
        if kind == OfficeUpgradeKind::ExtraWorkstation {
            employees.max_employees = 2;
        }
        Ok(())
    }
}

#[derive(Message, Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum OfficeUpgradeCommand {
    Buy(OfficeUpgradeKind),
}

#[derive(Component)]
pub struct PurchasedOfficeProp;

pub fn rebuild_colliders(colliders: &mut OfficeColliders, expansion: &OfficeExpansionState) {
    colliders.room_min_x = -2.68;
    if colliders.room_max_x < 2.68 {
        colliders.room_max_x = 2.68;
    }
    colliders.room_half_z = 2.18;
    colliders.obstacles = vec![
        // Founder desk + chair area
        Aabb2d {
            min_x: -2.05,
            max_x: -0.55,
            min_z: -2.05,
            max_z: -0.60,
        },
        // Developer #1 desk + chair area
        Aabb2d {
            min_x: 0.55,
            max_x: 2.05,
            min_z: -2.05,
            max_z: -0.60,
        },
        // West wall wooden bookshelf
        Aabb2d {
            min_x: -2.95,
            max_x: -2.52,
            min_z: -1.25,
            max_z: -0.05,
        },
    ];
    for kind in &expansion.purchased {
        colliders.obstacles.push(kind.collider());
    }
    if colliders.room_max_x > 3.0 {
        colliders.obstacles.push(Aabb2d {
            min_x: 3.30,
            max_x: 5.55,
            min_z: -2.30,
            max_z: -0.25,
        });
        colliders.obstacles.push(Aabb2d {
            min_x: 4.10,
            max_x: 5.35,
            min_z: 0.85,
            max_z: 1.95,
        });
    }
    if colliders.room_max_x > 8.0 {
        colliders.obstacles.push(Aabb2d {
            min_x: 7.10,
            max_x: 10.20,
            min_z: -2.05,
            max_z: -0.60,
        });
        colliders.obstacles.push(Aabb2d {
            min_x: 8.05,
            max_x: 9.45,
            min_z: 0.55,
            max_z: 1.45,
        });
    }
}

pub fn spawn_upgrade_visual(
    commands: &mut Commands,
    asset_server: &AssetServer,
    kind: OfficeUpgradeKind,
) {
    match kind {
        OfficeUpgradeKind::OfficePlant => {
            commands.spawn((
                PurchasedOfficeProp,
                WorldAssetRoot(
                    asset_server.load(
                        GltfAssetLabel::Scene(0).from_asset("furniture/furniture_plant_pot.glb"),
                    ),
                ),
                Transform::from_xyz(2.55, 0.0, -2.05),
            ));
        }
        OfficeUpgradeKind::Whiteboard => {
            commands.spawn((
                PurchasedOfficeProp,
                WorldAssetRoot(
                    asset_server.load(
                        GltfAssetLabel::Scene(0).from_asset("furniture/furniture_whiteboard.glb"),
                    ),
                ),
                Transform::from_xyz(0.0, 0.0, -2.18),
            ));
        }
        OfficeUpgradeKind::LoungeSofa => {
            commands.spawn((
                PurchasedOfficeProp,
                WorldAssetRoot(
                    asset_server.load(
                        GltfAssetLabel::Scene(0).from_asset("furniture/furniture_sofa_basic.glb"),
                    ),
                ),
                Transform::from_xyz(2.10, 0.0, 1.50)
                    .with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
                Interactable {
                    kind: InteractableKind::Sofa,
                    prompt: "Istirahat di Sofa (+Energi)".into(),
                    range: 2.3,
                    offset: Vec3::new(0.0, 0.5, 0.0),
                },
            ));
        }
        OfficeUpgradeKind::ServerRack => {
            commands.spawn((
                PurchasedOfficeProp,
                WorldAssetRoot(
                    asset_server.load(
                        GltfAssetLabel::Scene(0).from_asset("equipment/equipment_server_rack.glb"),
                    ),
                ),
                Transform::from_xyz(-2.45, 0.0, 2.00)
                    .with_rotation(Quat::from_rotation_y(FRAC_PI_2 * 0.5)),
            ));
        }
        OfficeUpgradeKind::ExtraWorkstation => {
            let root = spawn_workstation(
                commands,
                asset_server,
                Vec3::new(-2.20, 0.0, 0.60),
                FRAC_PI_2,
                false,
            );
            commands.entity(root).insert(PurchasedOfficeProp);
        }
    }
}

fn spawn_workstation(
    commands: &mut Commands,
    asset_server: &AssetServer,
    desk_pos: Vec3,
    yaw: f32,
    interactable: bool,
) -> Entity {
    let mut root = commands.spawn((
        Transform::from_translation(desk_pos).with_rotation(Quat::from_rotation_y(yaw)),
        Visibility::default(),
    ));
    if interactable {
        root.insert(Interactable {
            kind: InteractableKind::Workstation,
            prompt: "Gunakan Workstation".into(),
            range: 2.4,
            offset: Vec3::new(0.0, 0.95, 0.0),
        });
    }

    root.with_children(|parent| {
        parent.spawn((
            WorldAssetRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("furniture/furniture_desk_basic.glb"),
                ),
            ),
            Transform::IDENTITY,
        ));
        parent.spawn((
            WorldAssetRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("furniture/furniture_chair_office.glb"),
                ),
            ),
            Transform::from_xyz(0.0, 0.0, 0.83),
        ));
        parent.spawn((
            WorldAssetRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("equipment/equipment_monitor_basic.glb"),
                ),
            ),
            Transform::from_xyz(0.0, 0.75, -0.17),
        ));
        parent.spawn((
            WorldAssetRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("equipment/equipment_keyboard_mouse.glb"),
                ),
            ),
            Transform::from_xyz(0.0, 0.75, 0.13),
        ));
        parent.spawn((
            WorldAssetRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("equipment/equipment_pc_basic.glb"),
                ),
            ),
            Transform::from_xyz(0.52, 0.75, -0.07),
        ));
    });

    root.id()
}

fn setup_office_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    expansion: Res<OfficeExpansionState>,
    mut colliders: ResMut<OfficeColliders>,
) {
    // Outdoor warm courtyard lawn so window views feel cozy & sunlit
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(56.0, 56.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.54, 0.63, 0.41),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.07, 0.0),
    ));

    // Perimeter warm oak ceiling beams (leaves central skylight open for golden sunbeams)
    let beam_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.68, 0.52, 0.36),
        perceptual_roughness: 0.72,
        ..default()
    });
    for (pos, size) in [
        (Vec3::new(0.0, 2.84, -2.1), Vec3::new(6.15, 0.08, 0.95)),
        (Vec3::new(0.0, 2.84, 2.1), Vec3::new(6.15, 0.08, 0.95)),
        (Vec3::new(-2.55, 2.84, 0.0), Vec3::new(1.05, 0.08, 3.25)),
        (Vec3::new(2.55, 2.84, 0.0), Vec3::new(1.05, 0.08, 3.25)),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(beam_mat.clone()),
            Transform::from_translation(pos),
        ));
    }
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

    // North wall (z = -2.5)
    for gx in 0..6 {
        let x = -2.5 + gx as f32;
        let scene = if gx == 1 || gx == 4 {
            wall_window.clone()
        } else {
            wall_plain.clone()
        };
        commands.spawn((WorldAssetRoot(scene), Transform::from_xyz(x, 0.0, -2.5)));
    }

    // South wall (z = +2.5): door module at gx = 3 (x = +0.5)
    for gx in 0..6 {
        let x = -2.5 + gx as f32;
        if gx == 3 {
            commands.spawn((
                WorldAssetRoot(wall_door.clone()),
                Transform::from_xyz(x, 0.0, 2.5),
            ));
            let hinge_pos = Vec3::new(x - 0.40, 0.0, 2.5);
            commands.spawn((
                WorldAssetRoot(door_handle.clone()),
                Transform::from_translation(hinge_pos),
                OfficeDoor {
                    is_open: false,
                    current_angle: 0.0,
                    target_angle: 0.0,
                },
                Interactable {
                    kind: InteractableKind::Door,
                    prompt: "Buka Pintu".into(),
                    range: 2.3,
                    offset: Vec3::new(0.40, 1.0, 0.0),
                },
            ));
        } else {
            commands.spawn((
                WorldAssetRoot(wall_plain.clone()),
                Transform::from_xyz(x, 0.0, 2.5),
            ));
        }
    }

    // West & East walls
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
        let mut east_cmd = commands.spawn((
            WorldAssetRoot(east_scene),
            Transform::from_xyz(3.0, 0.0, z).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        ));
        if gz == 1 || gz == 2 || gz == 3 {
            east_cmd.insert(EastArchwayTile);
        }
    }

    // Founder Workstation (left) & Developer #1 Workstation (right)
    spawn_workstation(&mut commands, &asset_server, Vec3::new(-1.3, 0.0, -1.65), 0.0, true);
    spawn_workstation(&mut commands, &asset_server, Vec3::new(1.3, 0.0, -1.65), 0.0, false);

    // Golden-hour sunlight angled across the tall grid windows + warm interior fill lamp
    commands.spawn((
        SunLight,
        DirectionalLight {
            color: Color::srgb(1.0, 0.84, 0.60),
            illuminance: 11_500.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(7.2, 4.2, -4.6).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        WarmLampLight,
        PointLight {
            color: Color::srgb(1.0, 0.88, 0.70),
            intensity: 150_000.0,
            range: 14.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 2.48, -0.15),
    ));

    rebuild_colliders(&mut colliders, &expansion);
}

fn handle_office_upgrade_commands(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut reader: MessageReader<OfficeUpgradeCommand>,
    clock: Res<GameClock>,
    mut expansion: ResMut<OfficeExpansionState>,
    mut finance: ResMut<FinanceState>,
    mut company: ResMut<CompanyState>,
    mut employees: ResMut<EmployeeState>,
    mut colliders: ResMut<OfficeColliders>,
    mut notify: MessageWriter<NotifyMessage>,
) {
    for cmd in reader.read() {
        match cmd {
            OfficeUpgradeCommand::Buy(kind) => {
                match expansion.buy_upgrade(
                    *kind,
                    clock.day,
                    &mut finance,
                    &mut company,
                    &mut employees,
                ) {
                    Ok(()) => {
                        spawn_upgrade_visual(&mut commands, &asset_server, *kind);
                        rebuild_colliders(&mut colliders, &expansion);
                        notify.write(NotifyMessage::info(format!(
                            "Dipasang di kantor: {} ({})",
                            kind.label(),
                            kind.effect_summary()
                        )));
                    }
                    Err(err) => {
                        notify.write(NotifyMessage::warn(err));
                    }
                }
            }
        }
    }
}

fn handle_door_interactions(
    mut reader: MessageReader<InteractTriggered>,
    mut door_q: Query<(&mut OfficeDoor, &mut Interactable)>,
) {
    for ev in reader.read() {
        if ev.kind != InteractableKind::Door {
            continue;
        }
        if let Ok((mut door, mut interactable)) = door_q.get_mut(ev.entity) {
            door.is_open = !door.is_open;
            door.target_angle = if door.is_open { -1.45 } else { 0.0 };
            interactable.prompt = if door.is_open {
                "Tutup Pintu".into()
            } else {
                "Buka Pintu".into()
            };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buying_extra_workstation_unlocks_second_employee_slot() {
        let mut exp = OfficeExpansionState::default();
        let mut fin = FinanceState::default();
        let mut comp = CompanyState::default();
        let mut emps = EmployeeState::default();

        assert_eq!(emps.max_employees, 1);
        exp.buy_upgrade(
            OfficeUpgradeKind::ExtraWorkstation,
            1,
            &mut fin,
            &mut comp,
            &mut emps,
        )
        .unwrap();

        assert_eq!(emps.max_employees, 2);
        assert_eq!(fin.cash_balance, 25_000 - 1_200);
        assert!(exp.buy_upgrade(
            OfficeUpgradeKind::ExtraWorkstation,
            1,
            &mut fin,
            &mut comp,
            &mut emps,
        )
        .is_err());
    }
}
