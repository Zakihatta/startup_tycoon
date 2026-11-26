use bevy::prelude::*;
use std::{
    f32::consts::PI,
    time::Duration,
};

use crate::{
    company::CompanyState,
    core::{DayPassedMessage, GameClock},
    economy::{FinanceState, TransactionKind},
    product::{ProductPhase, ProductState},
};

const DEVELOPER_ASSET: &str = "characters/employees/character_developer.glb";

pub struct EmployeePlugin;

impl Plugin for EmployeePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EmployeeState>()
            .add_message::<EmployeeCommand>()
            .add_systems(Startup, setup_developer_anim_graph)
            .add_systems(
                Update,
                (
                    handle_employee_commands,
                    move_developer_npcs,
                    setup_developer_animations,
                    sync_developer_animations,
                    sync_employee_productivity,
                    charge_daily_salaries,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmployeeWorkState {
    WalkingToDesk,
    Working,
    SittingIdle,
}

impl EmployeeWorkState {
    pub fn label(self) -> &'static str {
        match self {
            EmployeeWorkState::WalkingToDesk => "Berjalan ke Meja",
            EmployeeWorkState::Working => "Sedang Coding",
            EmployeeWorkState::SittingIdle => "Duduk Santai",
        }
    }
}

#[derive(Clone, Debug)]
pub struct HireCandidate {
    pub id: u32,
    pub name: String,
    pub skill: f32,
    pub daily_salary: i64,
    pub hiring_fee: i64,
    pub portrait_path: &'static str,
}

#[derive(Clone, Debug)]
pub struct EmployeeProfile {
    pub id: u32,
    pub name: String,
    pub role: &'static str,
    pub skill: f32,
    pub daily_salary: i64,
    pub portrait_path: &'static str,
    pub state: EmployeeWorkState,
}

#[derive(Resource, Debug)]
pub struct EmployeeState {
    pub candidates: Vec<HireCandidate>,
    pub roster: Vec<EmployeeProfile>,
    pub max_employees: usize,
}

impl Default for EmployeeState {
    fn default() -> Self {
        Self {
            candidates: vec![
                HireCandidate {
                    id: 1,
                    name: "Raka Pratama".to_string(),
                    skill: 1.2,
                    daily_salary: 160,
                    hiring_fee: 600,
                    portrait_path: "characters/employees/portraits/developer_01.png",
                },
                HireCandidate {
                    id: 2,
                    name: "Sinta Wijaya".to_string(),
                    skill: 1.8,
                    daily_salary: 240,
                    hiring_fee: 1_000,
                    portrait_path: "characters/employees/portraits/developer_02.png",
                },
            ],
            roster: Vec::new(),
            max_employees: 1,
        }
    }
}

impl EmployeeState {
    pub fn hire_candidate(
        &mut self,
        candidate_idx: usize,
        day: u32,
        finance: &mut FinanceState,
        company: &mut CompanyState,
    ) -> Result<EmployeeProfile, &'static str> {
        if self.roster.len() >= self.max_employees {
            return Err("Kapasitas workstation karyawan sudah penuh.");
        }
        let cand = self
            .candidates
            .get(candidate_idx)
            .ok_or("Kandidat tidak ditemukan.")?
            .clone();
        if finance.cash_balance < cand.hiring_fee {
            return Err("Kas tidak cukup untuk biaya rekrutmen.");
        }

        finance.record_transaction(
            day,
            format!("Rekrut Dev: {}", cand.name),
            -cand.hiring_fee,
            TransactionKind::Salary,
        );

        let profile = EmployeeProfile {
            id: cand.id,
            name: cand.name,
            role: "Developer",
            skill: cand.skill,
            daily_salary: cand.daily_salary,
            portrait_path: cand.portrait_path,
            state: EmployeeWorkState::WalkingToDesk,
        };
        self.roster.push(profile.clone());
        company.team_size = 1 + self.roster.len() as u32;
        Ok(profile)
    }

    pub fn productivity_bonus(&self) -> f32 {
        self.roster
            .iter()
            .filter(|e| e.state == EmployeeWorkState::Working)
            .map(|e| e.skill * 1.8)
            .sum()
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub enum EmployeeCommand {
    HireCandidate(usize),
}

#[derive(Component)]
pub struct DeveloperNpc {
    pub employee_id: u32,
    pub waypoints: Vec<Vec3>,
    pub wp_index: usize,
    pub seated: bool,
    pub facing_yaw: f32,
    pub active_anim: Option<usize>,
}

#[derive(Component)]
pub struct DeveloperAnimPlayer;

#[derive(Resource)]
struct DeveloperAnimations {
    graph: Handle<AnimationGraph>,
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    sitting: AnimationNodeIndex,
    typing: AnimationNodeIndex,
}

fn setup_developer_anim_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let clips = [
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(DEVELOPER_ASSET)),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(DEVELOPER_ASSET)),
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(DEVELOPER_ASSET)),
        asset_server.load(GltfAssetLabel::Animation(3).from_asset(DEVELOPER_ASSET)),
    ];
    let (graph, nodes) = AnimationGraph::from_clips(clips);
    let graph_handle = graphs.add(graph);

    commands.insert_resource(DeveloperAnimations {
        graph: graph_handle,
        idle: nodes[0],
        walk: nodes[1],
        sitting: nodes[2],
        typing: nodes[3],
    });
}

fn handle_employee_commands(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut reader: MessageReader<EmployeeCommand>,
    clock: Res<GameClock>,
    mut employees: ResMut<EmployeeState>,
    mut finance: ResMut<FinanceState>,
    mut company: ResMut<CompanyState>,
) {
    for cmd in reader.read() {
        match cmd {
            EmployeeCommand::HireCandidate(idx) => {
                if let Ok(profile) =
                    employees.hire_candidate(*idx, clock.day, &mut finance, &mut company)
                {
                    commands.spawn((
                        Transform::from_xyz(0.5, 0.0, 1.85),
                        Visibility::default(),
                        DeveloperNpc {
                            employee_id: profile.id,
                            waypoints: vec![
                                Vec3::new(0.5, 0.0, 0.3),
                                Vec3::new(1.3, 0.0, -0.35),
                                Vec3::new(1.3, 0.0, -0.82),
                            ],
                            wp_index: 0,
                            seated: false,
                            facing_yaw: 0.0,
                            active_anim: None,
                        },
                        WorldAssetRoot(
                            asset_server
                                .load(GltfAssetLabel::Scene(0).from_asset(DEVELOPER_ASSET)),
                        ),
                    ));
                }
            }
        }
    }
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

fn move_developer_npcs(
    time: Res<Time>,
    clock: Res<GameClock>,
    product: Res<ProductState>,
    mut employees: ResMut<EmployeeState>,
    mut npc_q: Query<(&mut Transform, &mut DeveloperNpc)>,
) {
    let speed_mult = clock.speed.multiplier();
    let dt = time.delta_secs() * speed_mult;
    let is_building = product.active.as_ref().is_some_and(|p| {
        matches!(p.phase, ProductPhase::Development | ProductPhase::Testing)
    });

    for (mut tf, mut npc) in &mut npc_q {
        if !npc.seated {
            if let Some(&target) = npc.waypoints.get(npc.wp_index) {
                let to_target = target - tf.translation;
                let dist = Vec2::new(to_target.x, to_target.z).length();
                if dist < 0.06 {
                    tf.translation = target;
                    npc.wp_index += 1;
                    if npc.wp_index >= npc.waypoints.len() {
                        npc.seated = true;
                        npc.facing_yaw = 0.0; // Faces -Z toward desk & monitor
                        tf.rotation = Quat::IDENTITY;
                    }
                } else if dt > 0.0 {
                    let dir = Vec3::new(to_target.x, 0.0, to_target.z).normalize();
                    let step = (1.85 * dt).min(dist);
                    tf.translation += dir * step;

                    let target_yaw = (-dir.x).atan2(-dir.z);
                    let diff = shortest_angle_diff(npc.facing_yaw, target_yaw);
                    npc.facing_yaw += diff * (10.0 * time.delta_secs()).min(1.0);
                    tf.rotation = Quat::from_rotation_y(npc.facing_yaw);
                }
            }
        }

        if let Some(emp) = employees
            .roster
            .iter_mut()
            .find(|e| e.id == npc.employee_id)
        {
            emp.state = if !npc.seated {
                EmployeeWorkState::WalkingToDesk
            } else if is_building {
                EmployeeWorkState::Working
            } else {
                EmployeeWorkState::SittingIdle
            };
        }
    }
}

fn has_developer_ancestor(
    mut current: Entity,
    parents: &Query<&ChildOf>,
    dev_npcs: &Query<(), With<DeveloperNpc>>,
) -> Option<Entity> {
    loop {
        if dev_npcs.contains(current) {
            return Some(current);
        }
        match parents.get(current) {
            Ok(child_of) => current = child_of.parent(),
            Err(_) => return None,
        }
    }
}

fn setup_developer_animations(
    mut commands: Commands,
    anims: Res<DeveloperAnimations>,
    parents: Query<&ChildOf>,
    dev_npcs: Query<(), With<DeveloperNpc>>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        if has_developer_ancestor(entity, &parents, &dev_npcs).is_some() {
            let mut transitions = AnimationTransitions::new();
            transitions
                .play(&mut player, anims.walk, Duration::ZERO)
                .repeat();

            commands
                .entity(entity)
                .insert((
                    DeveloperAnimPlayer,
                    AnimationGraphHandle(anims.graph.clone()),
                    transitions,
                ));
        }
    }
}

fn sync_developer_animations(
    clock: Res<GameClock>,
    product: Res<ProductState>,
    anims: Res<DeveloperAnimations>,
    parents: Query<&ChildOf>,
    mut npc_q: Query<&mut DeveloperNpc>,
    mut anim_q: Query<
        (Entity, &mut AnimationPlayer, &mut AnimationTransitions),
        With<DeveloperAnimPlayer>,
    >,
) {
    let is_building = product.active.as_ref().is_some_and(|p| {
        matches!(p.phase, ProductPhase::Development | ProductPhase::Testing)
    });

    for (anim_entity, mut anim_player, mut transitions) in &mut anim_q {
        // Walk up hierarchy to find owning DeveloperNpc
        let mut cur = anim_entity;
        let mut owner_npc = None;
        while let Ok(child_of) = parents.get(cur) {
            cur = child_of.parent();
            if npc_q.contains(cur) {
                owner_npc = Some(cur);
                break;
            }
        }
        let Some(npc_entity) = owner_npc else {
            continue;
        };
        let Ok(mut npc) = npc_q.get_mut(npc_entity) else {
            continue;
        };

        let target_idx = if !npc.seated {
            if clock.speed.multiplier() <= 0.0 {
                0 // Idle while paused mid-walk
            } else {
                1 // Walk
            }
        } else if is_building {
            3 // Typing
        } else {
            2 // Sitting
        };

        if npc.active_anim != Some(target_idx) {
            let node = match target_idx {
                0 => anims.idle,
                1 => anims.walk,
                2 => anims.sitting,
                _ => anims.typing,
            };
            transitions
                .play(&mut anim_player, node, Duration::from_millis(180))
                .repeat();
            npc.active_anim = Some(target_idx);
        }

        let anim_speed = if clock.speed.multiplier() <= 0.0 && !npc.seated {
            0.0
        } else if target_idx == 3 && clock.speed == crate::core::SimSpeed::Fast {
            1.6
        } else if target_idx == 1 && clock.speed == crate::core::SimSpeed::Fast {
            1.6
        } else {
            1.0
        };
        if let Some(active) = anim_player.playing_animations_mut().next() {
            active.1.set_speed(anim_speed);
        }
    }
}

fn sync_employee_productivity(
    employees: Res<EmployeeState>,
    mut product: ResMut<ProductState>,
) {
    let bonus = employees.productivity_bonus();
    if (product.total_productivity_bonus - bonus).abs() > 1e-4 {
        product.total_productivity_bonus = bonus;
    }
}

fn charge_daily_salaries(
    mut day_reader: MessageReader<DayPassedMessage>,
    employees: Res<EmployeeState>,
    mut finance: ResMut<FinanceState>,
) {
    for ev in day_reader.read() {
        for emp in &employees.roster {
            finance.record_transaction(
                ev.new_day,
                format!("Gaji {}: {}", emp.role, emp.name),
                -emp.daily_salary,
                TransactionKind::Salary,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hiring_deducts_fee_and_updates_team_size() {
        let mut emps = EmployeeState::default();
        let mut fin = FinanceState::default();
        let mut comp = CompanyState::default();

        let hired = emps.hire_candidate(0, 1, &mut fin, &mut comp).unwrap();
        assert_eq!(hired.name, "Raka Pratama");
        assert_eq!(fin.cash_balance, 25_000 - 600);
        assert_eq!(comp.team_size, 2);
        assert_eq!(emps.roster.len(), 1);
    }

    #[test]
    fn working_developer_adds_productivity_bonus() {
        let mut emps = EmployeeState::default();
        let mut fin = FinanceState::default();
        let mut comp = CompanyState::default();

        emps.hire_candidate(1, 1, &mut fin, &mut comp).unwrap();
        assert_eq!(emps.productivity_bonus(), 0.0);

        emps.roster[0].state = EmployeeWorkState::Working;
        assert!((emps.productivity_bonus() - 1.8 * 1.8).abs() < 1e-4);
    }
}
