use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

use crate::{
    core::GameClock,
    interaction::{Interactable, InteractableKind},
    life::{CompanyStage, CompanyStageState},
    office::{Aabb2d, EastArchwayTile, OfficeColliders, SunLight, WarmLampLight},
};

pub struct OfficeAtmospherePlugin;

impl Plugin for OfficeAtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoftVisualState>()
            .add_systems(Startup, setup_studio_details_and_motes)
            .add_systems(
                Update,
                (
                    animate_dust_motes,
                    sync_ambient_lighting,
                    sync_stage_2_loft,
                    sync_stage_3_lobby,
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct LoftVisualState {
    pub stage_2_spawned: bool,
    pub stage_3_spawned: bool,
}

#[derive(Component)]
pub struct Stage2EastArchwayTile;

#[derive(Component)]
pub struct Stage3LobbyProp;

#[derive(Component)]
pub struct DustMote {
    base_pos: Vec3,
    phase: f32,
    speed: f32,
}

#[derive(Component)]
pub struct Stage2LoftProp;

fn spawn_bookshelf(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pos: Vec3,
    yaw: f32,
    is_stage_2: bool,
) {
    let wood_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.56, 0.39, 0.25),
        perceptual_roughness: 0.68,
        ..default()
    });
    let book_colors = [
        Color::srgb(0.82, 0.36, 0.28),
        Color::srgb(0.26, 0.56, 0.66),
        Color::srgb(0.92, 0.74, 0.30),
        Color::srgb(0.36, 0.62, 0.42),
        Color::srgb(0.76, 0.68, 0.58),
    ];
    let book_mats: Vec<Handle<StandardMaterial>> = book_colors
        .into_iter()
        .map(|c| {
            materials.add(StandardMaterial {
                base_color: c,
                perceptual_roughness: 0.78,
                ..default()
            })
        })
        .collect();

    let mut entity = commands.spawn((
        Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)),
        Visibility::default(),
    ));
    if is_stage_2 {
        entity.insert(Stage2LoftProp);
    }

    entity.with_children(|root| {
        // Frame & shelves (1.10m wide x 1.75m tall x 0.32m deep)
        root.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.10, 1.75, 0.04))),
            MeshMaterial3d(wood_mat.clone()),
            Transform::from_xyz(0.0, 0.875, -0.14),
        ));
        for sx in [-0.53, 0.53] {
            root.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.04, 1.75, 0.32))),
                MeshMaterial3d(wood_mat.clone()),
                Transform::from_xyz(sx, 0.875, 0.0),
            ));
        }
        for sy in [0.06, 0.52, 0.96, 1.40, 1.73] {
            root.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.06, 0.035, 0.31))),
                MeshMaterial3d(wood_mat.clone()),
                Transform::from_xyz(0.0, sy, 0.0),
            ));
        }
        // Colorful book rows
        for (row_idx, sy) in [0.54, 0.98, 1.42].into_iter().enumerate() {
            for b in 0..7 {
                let bx = -0.38 + (b as f32) * 0.12;
                let bh = 0.24 + ((b + row_idx) % 3) as f32 * 0.04;
                root.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.085, bh, 0.20))),
                    MeshMaterial3d(book_mats[(b + row_idx) % book_mats.len()].clone()),
                    Transform::from_xyz(bx, sy + bh * 0.5, 0.02),
                ));
            }
        }
    });
}

fn spawn_hanging_ivy(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pos: Vec3,
    is_stage_2: bool,
) {
    let leaf_light = materials.add(StandardMaterial {
        base_color: Color::srgb(0.46, 0.66, 0.32),
        perceptual_roughness: 0.74,
        ..default()
    });
    let leaf_dark = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.52, 0.24),
        perceptual_roughness: 0.78,
        ..default()
    });

    let mut entity = commands.spawn((Transform::from_translation(pos), Visibility::default()));
    if is_stage_2 {
        entity.insert(Stage2LoftProp);
    }

    entity.with_children(|root| {
        for (idx, offset, radius, dark) in [
            (0, Vec3::new(0.0, 0.0, 0.0), 0.16, false),
            (1, Vec3::new(-0.18, -0.14, 0.06), 0.14, true),
            (2, Vec3::new(0.16, -0.22, -0.04), 0.15, false),
            (3, Vec3::new(-0.05, -0.38, 0.05), 0.13, true),
            (4, Vec3::new(0.10, -0.52, 0.0), 0.11, false),
        ] {
            let _ = idx;
            root.spawn((
                Mesh3d(meshes.add(Sphere::new(radius))),
                MeshMaterial3d(if dark { leaf_dark.clone() } else { leaf_light.clone() }),
                Transform::from_translation(offset).with_scale(Vec3::new(1.15, 0.85, 1.0)),
            ));
        }
    });
}

fn setup_studio_details_and_motes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // West wall built-in wooden bookshelf
    spawn_bookshelf(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-2.74, 0.0, -0.65),
        FRAC_PI_2,
        false,
    );

    // West wall minimalist geometric startup sign (echoing Reference 2's S-art plaque)
    let plaque_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.93, 0.89),
        perceptual_roughness: 0.72,
        ..default()
    });
    let ink_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.17),
        perceptual_roughness: 0.55,
        ..default()
    });
    let dot_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.38, 0.28),
        perceptual_roughness: 0.45,
        ..default()
    });
    commands
        .spawn((
            Transform::from_xyz(-2.91, 1.62, 0.82).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
            Visibility::default(),
        ))
        .with_children(|art| {
            art.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.68, 0.68, 0.03))),
                MeshMaterial3d(plaque_mat),
            ));
            for (y, w, x) in [
                (0.16, 0.36, 0.0),
                (0.0, 0.36, 0.0),
                (-0.16, 0.36, 0.0),
            ] {
                art.spawn((
                    Mesh3d(meshes.add(Cuboid::new(w, 0.045, 0.04))),
                    MeshMaterial3d(ink_mat.clone()),
                    Transform::from_xyz(x, y, 0.006),
                ));
            }
            art.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.045, 0.20, 0.04))),
                MeshMaterial3d(ink_mat.clone()),
                Transform::from_xyz(-0.16, 0.08, 0.006),
            ));
            art.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.045, 0.20, 0.04))),
                MeshMaterial3d(ink_mat),
                Transform::from_xyz(0.16, -0.08, 0.006),
            ));
            art.spawn((
                Mesh3d(meshes.add(Sphere::new(0.032))),
                MeshMaterial3d(dot_mat),
                Transform::from_xyz(0.20, 0.20, 0.015),
            ));
        });

    // Warm pendant light fixture above center
    let lamp_cord = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.20, 0.22),
        perceptual_roughness: 0.60,
        ..default()
    });
    let lamp_glow = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.92, 0.76),
        emissive: LinearRgba::new(1.4, 1.05, 0.60, 1.0),
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.014, 0.45))),
        MeshMaterial3d(lamp_cord),
        Transform::from_xyz(0.0, 2.74, -0.15),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.36, 0.05, 0.36))),
        MeshMaterial3d(lamp_glow),
        Transform::from_xyz(0.0, 2.50, -0.15),
    ));

    // Hanging biophilic ivy along upper north beam
    spawn_hanging_ivy(&mut commands, &mut meshes, &mut materials, Vec3::new(-0.05, 2.66, -2.30), false);
    spawn_hanging_ivy(&mut commands, &mut meshes, &mut materials, Vec3::new(2.25, 2.66, -2.30), false);

    // Stylized outdoor trees outside the windows so exterior vista looks cozy
    let trunk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.44, 0.31, 0.21),
        perceptual_roughness: 0.88,
        ..default()
    });
    let tree_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.64, 0.36),
        perceptual_roughness: 0.80,
        ..default()
    });
    for (tx, tz, scale) in [
        (-2.8, -5.8, 1.0),
        (1.6, -6.5, 1.25),
        (6.8, -4.6, 1.1),
        (7.5, 1.8, 0.95),
        (-5.6, 1.2, 1.15),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.16 * scale, 1.6 * scale))),
            MeshMaterial3d(trunk_mat.clone()),
            Transform::from_xyz(tx, 0.8 * scale, tz),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.95 * scale))),
            MeshMaterial3d(tree_mat.clone()),
            Transform::from_xyz(tx, 2.0 * scale, tz),
        ));
    }

    // Floating warm golden dust motes drifting inside sunbeam area
    let mote_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.93, 0.72),
        emissive: LinearRgba::new(1.5, 1.15, 0.55, 1.0),
        ..default()
    });
    let mote_mesh = meshes.add(Sphere::new(0.015));
    for i in 0..24 {
        let fi = i as f32;
        let base = Vec3::new(
            -2.1 + ((i * 7) % 13) as f32 * 0.35,
            0.55 + ((i * 5) % 9) as f32 * 0.22,
            -1.9 + ((i * 11) % 11) as f32 * 0.34,
        );
        commands.spawn((
            DustMote {
                base_pos: base,
                phase: fi * 0.85,
                speed: 0.75 + (i % 4) as f32 * 0.18,
            },
            Mesh3d(mote_mesh.clone()),
            MeshMaterial3d(mote_mat.clone()),
            Transform::from_translation(base),
        ));
    }
}

fn animate_dust_motes(time: Res<Time>, mut motes_q: Query<(&DustMote, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (mote, mut tf) in &mut motes_q {
        let p = t * mote.speed + mote.phase;
        tf.translation = mote.base_pos
            + Vec3::new(
                0.08 * (p * 0.8).sin(),
                0.12 * p.sin(),
                0.08 * (p * 0.6).cos(),
            );
    }
}

fn sync_ambient_lighting(
    clock: Res<GameClock>,
    mut sun_q: Query<(&mut DirectionalLight, &mut Transform), (With<SunLight>, Without<WarmLampLight>)>,
    mut lamp_q: Query<&mut PointLight, (With<WarmLampLight>, Without<SunLight>)>,
) {
    if !clock.is_changed() {
        return;
    }

    let hour_f = clock.hour as f32 + (clock.minute as f32) / 60.0;
    // Golden afternoon default peak around 14:00..17:30 matching Reference 2
    let (illuminance, sun_color, lamp_intensity, elev_y) = if (7.0..11.0).contains(&hour_f) {
        (10_500.0, Color::srgb(1.0, 0.89, 0.74), 130_000.0, 5.5)
    } else if (11.0..15.0).contains(&hour_f) {
        (11_500.0, Color::srgb(1.0, 0.91, 0.76), 120_000.0, 6.2)
    } else if (15.0..18.5).contains(&hour_f) {
        // Warm golden hour casting dramatic window-grid shadows
        (12_000.0, Color::srgb(1.0, 0.78, 0.52), 185_000.0, 3.8)
    } else if (18.5..20.5).contains(&hour_f) {
        // Twilight dusk
        (3_200.0, Color::srgb(0.74, 0.74, 0.92), 250_000.0, 3.2)
    } else {
        // Cozy late-night lamp studio
        (1_400.0, Color::srgb(0.56, 0.64, 0.86), 280_000.0, 5.0)
    };

    for (mut light, mut tf) in &mut sun_q {
        light.illuminance = illuminance;
        light.color = sun_color;
        *tf = Transform::from_xyz(7.2, elev_y, -4.6).looking_at(Vec3::ZERO, Vec3::Y);
    }

    for mut lamp in &mut lamp_q {
        lamp.intensity = lamp_intensity;
    }
}

#[allow(clippy::too_many_arguments)]
fn sync_stage_2_loft(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    stage: Res<CompanyStageState>,
    mut loft_state: ResMut<LoftVisualState>,
    mut colliders: ResMut<OfficeColliders>,
    mut arch_q: Query<&mut Visibility, With<EastArchwayTile>>,
    loft_prop_q: Query<Entity, With<Stage2LoftProp>>,
) {
    let should_have_loft = stage.current != CompanyStage::Founder;
    if !should_have_loft && loft_state.stage_2_spawned {
        loft_state.stage_2_spawned = false;
        colliders.room_max_x = 2.68;
        for mut vis in &mut arch_q {
            *vis = Visibility::Inherited;
        }
        for ent in &loft_prop_q {
            commands.entity(ent).despawn();
        }
        return;
    }
    if !should_have_loft || loft_state.stage_2_spawned {
        return;
    }
    loft_state.stage_2_spawned = true;

    // Open the middle east wall panels into an archway connecting Ground Studio -> Stage 2 Loft
    for mut vis in &mut arch_q {
        *vis = Visibility::Hidden;
    }

    // Expand horizontal walking bounds into Stage 2 wing
    colliders.room_min_x = -2.68;
    colliders.room_max_x = 6.15;
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

    let floor_handle = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("environment/floor/environment_floor_tile.glb"),
    );
    let wall_plain = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("environment/walls/environment_wall_plain.glb"),
    );
    let wall_window = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("environment/walls/environment_wall_window.glb"),
    );

    // 4 x 5 oak floor tiles extending east (x = 3.5..6.5)
    for gx in 0..4 {
        for gz in 0..5 {
            commands.spawn((
                Stage2LoftProp,
                WorldAssetRoot(floor_handle.clone()),
                Transform::from_xyz(3.5 + gx as f32, 0.0, -2.0 + gz as f32),
            ));
        }
    }

    // Tall grid windows on North wall & East wall of Stage 2 Loft
    for gx in 0..4 {
        let x = 3.5 + gx as f32;
        commands.spawn((
            Stage2LoftProp,
            WorldAssetRoot(wall_window.clone()),
            Transform::from_xyz(x, 0.0, -2.5),
        ));
        commands.spawn((
            Stage2LoftProp,
            WorldAssetRoot(wall_plain.clone()),
            Transform::from_xyz(x, 0.0, 2.5),
        ));
    }
    for gz in 0..5 {
        let z = -2.0 + gz as f32;
        let east_scene = if gz <= 2 {
            wall_window.clone()
        } else {
            wall_plain.clone()
        };
        let mut cmd = commands.spawn((
            Stage2LoftProp,
            WorldAssetRoot(east_scene),
            Transform::from_xyz(7.0, 0.0, z).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        ));
        if gz >= 2 {
            cmd.insert(Stage2EastArchwayTile);
        }
    }

    let oak_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.62, 0.45, 0.30),
        perceptual_roughness: 0.65,
        ..default()
    });
    let rail_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.21, 0.23),
        perceptual_roughness: 0.45,
        metallic: 0.55,
        ..default()
    });
    let cushion_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.84, 0.72, 0.56),
        perceptual_roughness: 0.85,
        ..default()
    });
    let pillow_y = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.80, 0.32),
        perceptual_roughness: 0.85,
        ..default()
    });
    let pillow_b = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.56, 0.68),
        perceptual_roughness: 0.85,
        ..default()
    });

    // Wide wooden staircase rising from x=3.4 toward upper mezzanine x=5.2 (y = 0 -> 1.80)
    for step in 0..8 {
        let sx = 3.45 + step as f32 * 0.22;
        let sy = 0.11 + step as f32 * 0.21;
        commands.spawn((
            Stage2LoftProp,
            Mesh3d(meshes.add(Cuboid::new(0.26, 0.22, 1.35))),
            MeshMaterial3d(oak_mat.clone()),
            Transform::from_xyz(sx, sy, -1.45),
        ));
    }
    // Stair handrail posts
    commands.spawn((
        Stage2LoftProp,
        Mesh3d(meshes.add(Cuboid::new(1.85, 0.04, 0.04))),
        MeshMaterial3d(rail_mat.clone()),
        Transform::from_xyz(4.30, 1.78, -0.78).with_rotation(Quat::from_rotation_z(0.72)),
    ));

    // Upper Mezzanine Deck & Wood Fascia (y = 1.85, x = 5.1..6.9, z = -2.35..0.45)
    commands.spawn((
        Stage2LoftProp,
        Mesh3d(meshes.add(Cuboid::new(1.90, 0.24, 2.90))),
        MeshMaterial3d(oak_mat.clone()),
        Transform::from_xyz(5.95, 1.75, -0.95),
    ));
    // Mezzanine railing
    commands.spawn((
        Stage2LoftProp,
        Mesh3d(meshes.add(Cuboid::new(0.05, 0.55, 2.85))),
        MeshMaterial3d(oak_mat.clone()),
        Transform::from_xyz(5.02, 2.14, -0.95),
    ));

    // Upstairs Mezzanine Workstation & Bookshelf (just like Reference Image 2!)
    commands.spawn((
        Stage2LoftProp,
        WorldAssetRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("furniture/furniture_desk_basic.glb")),
        ),
        Transform::from_xyz(5.95, 1.87, -1.45).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn((
        Stage2LoftProp,
        WorldAssetRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("equipment/equipment_monitor_basic.glb")),
        ),
        Transform::from_xyz(6.05, 2.62, -1.72).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn((
        Stage2LoftProp,
        WorldAssetRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("equipment/equipment_monitor_basic.glb")),
        ),
        Transform::from_xyz(6.05, 2.62, -1.15).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    spawn_bookshelf(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(6.70, 1.87, -1.40),
        -FRAC_PI_2,
        true,
    );

    // Cascading hanging ivy draping down from the mezzanine beam
    spawn_hanging_ivy(&mut commands, &mut meshes, &mut materials, Vec3::new(5.00, 1.78, -0.35), true);
    spawn_hanging_ivy(&mut commands, &mut meshes, &mut materials, Vec3::new(5.00, 1.78, 0.25), true);

    // Under-stair / Cozy Reading Seating Nook with colorful pillows (Interactable Sofa Rest!)
    commands
        .spawn((
            Stage2LoftProp,
            Transform::from_xyz(4.70, 0.0, 1.40),
            Visibility::default(),
            Interactable {
                kind: InteractableKind::Sofa,
                prompt: "Istirahat di Sudut Santai (+Energi)".into(),
                range: 2.3,
                offset: Vec3::new(0.0, 0.45, 0.0),
            },
        ))
        .with_children(|nook| {
            nook.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.30, 0.36, 0.78))),
                MeshMaterial3d(oak_mat),
                Transform::from_xyz(0.0, 0.18, 0.0),
            ));
            nook.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.22, 0.10, 0.70))),
                MeshMaterial3d(cushion_mat),
                Transform::from_xyz(0.0, 0.41, 0.0),
            ));
            nook.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.24, 0.24, 0.12))),
                MeshMaterial3d(pillow_y),
                Transform::from_xyz(-0.36, 0.56, -0.18),
            ));
            nook.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.24, 0.24, 0.12))),
                MeshMaterial3d(pillow_b),
                Transform::from_xyz(0.36, 0.56, -0.18),
            ));
        });

    // Cozy ceramic potted plants framing the Stage 2 loft
    commands.spawn((
        Stage2LoftProp,
        WorldAssetRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("furniture/furniture_plant_pot.glb")),
        ),
        Transform::from_xyz(3.45, 0.0, -2.05),
    ));
    commands.spawn((
        Stage2LoftProp,
        WorldAssetRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("furniture/furniture_plant_pot.glb")),
        ),
        Transform::from_xyz(6.40, 0.0, 2.05),
    ));

    // Warm Loft Fill Light
    commands.spawn((
        Stage2LoftProp,
        WarmLampLight,
        PointLight {
            color: Color::srgb(1.0, 0.88, 0.70),
            intensity: 170_000.0,
            range: 14.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(5.2, 2.55, 0.0),
    ));
}

#[allow(clippy::too_many_arguments)]
fn sync_stage_3_lobby(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    stage: Res<CompanyStageState>,
    mut loft_state: ResMut<LoftVisualState>,
    mut colliders: ResMut<OfficeColliders>,
    mut arch2_q: Query<&mut Visibility, With<Stage2EastArchwayTile>>,
    lobby_prop_q: Query<Entity, With<Stage3LobbyProp>>,
) {
    let should_have_lobby = matches!(
        stage.current,
        CompanyStage::GrowingCompany | CompanyStage::Corporation
    );

    if !should_have_lobby && loft_state.stage_3_spawned {
        loft_state.stage_3_spawned = false;
        colliders.room_max_x = if loft_state.stage_2_spawned { 6.15 } else { 2.68 };
        for mut vis in &mut arch2_q {
            *vis = Visibility::Inherited;
        }
        for ent in &lobby_prop_q {
            commands.entity(ent).despawn();
        }
        return;
    }

    if !should_have_lobby || loft_state.stage_3_spawned {
        return;
    }
    loft_state.stage_3_spawned = true;

    for mut vis in &mut arch2_q {
        *vis = Visibility::Hidden;
    }

    colliders.room_max_x = 10.15;
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
    let desk_handle = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("furniture/furniture_desk_basic.glb"),
    );
    let chair_handle = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("furniture/furniture_chair_office.glb"),
    );
    let monitor_handle = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("equipment/equipment_monitor_basic.glb"),
    );
    let plant_handle = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("furniture/furniture_plant_pot.glb"),
    );

    // 4 x 5 oak floor wing (x = 7.5..10.5)
    for gx in 0..4 {
        for gz in 0..5 {
            commands.spawn((
                Stage3LobbyProp,
                WorldAssetRoot(floor_handle.clone()),
                Transform::from_xyz(7.5 + gx as f32, 0.0, -2.0 + gz as f32),
            ));
        }
    }

    // North tall grid windows, South lobby wall with open courier doorway at x = 8.5, and East windows
    for gx in 0..4 {
        let x = 7.5 + gx as f32;
        commands.spawn((
            Stage3LobbyProp,
            WorldAssetRoot(wall_window.clone()),
            Transform::from_xyz(x, 0.0, -2.5),
        ));
        let south_scene = if gx == 1 {
            wall_door.clone()
        } else {
            wall_plain.clone()
        };
        commands.spawn((
            Stage3LobbyProp,
            WorldAssetRoot(south_scene),
            Transform::from_xyz(x, 0.0, 2.5),
        ));
    }
    for gz in 0..5 {
        let z = -2.0 + gz as f32;
        let east_scene = if gz <= 2 {
            wall_window.clone()
        } else {
            wall_plain.clone()
        };
        commands.spawn((
            Stage3LobbyProp,
            WorldAssetRoot(east_scene),
            Transform::from_xyz(11.0, 0.0, z).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        ));
    }

    // Team Workstations #3, #4 (north) and Manager Desk #5 (south-west wing)
    for (pos, yaw) in [
        (Vec3::new(7.80, 0.0, -1.55), 0.0),
        (Vec3::new(9.50, 0.0, -1.55), 0.0),
        (Vec3::new(7.80, 0.0, 1.55), std::f32::consts::PI),
    ] {
        commands
            .spawn((
                Stage3LobbyProp,
                Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)),
                Visibility::default(),
            ))
            .with_children(|ws| {
                ws.spawn((WorldAssetRoot(desk_handle.clone()), Transform::IDENTITY));
                ws.spawn((
                    WorldAssetRoot(chair_handle.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.83),
                ));
                ws.spawn((
                    WorldAssetRoot(monitor_handle.clone()),
                    Transform::from_xyz(0.0, 0.75, -0.17),
                ));
            });
    }

    // Receptionist Lobby Counter facing south entrance (courier arrives at z=1.85)
    let oak_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.58, 0.42, 0.28),
        perceptual_roughness: 0.62,
        ..default()
    });
    let top_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.94, 0.91, 0.86),
        perceptual_roughness: 0.45,
        ..default()
    });
    commands
        .spawn((
            Stage3LobbyProp,
            Transform::from_xyz(8.75, 0.0, 1.00),
            Visibility::default(),
        ))
        .with_children(|rec| {
            rec.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.50, 0.96, 0.65))),
                MeshMaterial3d(oak_mat),
                Transform::from_xyz(0.0, 0.48, 0.0),
            ));
            rec.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.58, 0.06, 0.72))),
                MeshMaterial3d(top_mat),
                Transform::from_xyz(0.0, 0.99, 0.0),
            ));
        });

    commands.spawn((
        Stage3LobbyProp,
        WorldAssetRoot(plant_handle.clone()),
        Transform::from_xyz(10.35, 0.0, 1.95),
    ));
    commands.spawn((
        Stage3LobbyProp,
        WorldAssetRoot(plant_handle),
        Transform::from_xyz(7.25, 0.0, -2.05),
    ));

    // Warm Lobby Lighting
    commands.spawn((
        Stage3LobbyProp,
        WarmLampLight,
        PointLight {
            color: Color::srgb(1.0, 0.89, 0.72),
            intensity: 190_000.0,
            range: 15.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(8.9, 2.55, 0.1),
    ));
}
