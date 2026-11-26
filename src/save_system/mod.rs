use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    company::CompanyState,
    core::{GameClock, NotifyMessage},
    economy::FinanceState,
    employee::{
        spawn_developer_npc, DeveloperNpc, EmployeeProfile, EmployeeState, EmployeeWorkState,
    },
    interaction::Interactable,
    life::{
        ensure_employee_stats_sane, CompanyStageState, DeliveryState, PlayerStatus, QuestState,
    },
    market::MarketState,
    office::{
        rebuild_colliders, spawn_upgrade_visual, OfficeColliders, OfficeDoor,
        OfficeExpansionState, OfficeUpgradeKind, PurchasedOfficeProp,
    },
    player::PlayerController,
    product::ProductState,
};

pub const SAVE_SCHEMA_VERSION: u32 = 1;

pub struct SaveSystemPlugin;

impl Plugin for SaveSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveGameCommand>()
            .add_systems(Update, (handle_save_hotkeys, process_save_commands));
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub enum SaveGameCommand {
    Save,
    Load,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedClock {
    pub day: u32,
    pub hour: u8,
    pub minute: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveGameData {
    pub version: u32,
    pub clock: SavedClock,
    pub company: CompanyState,
    pub finance: FinanceState,
    pub product: ProductState,
    pub employees: Vec<EmployeeProfile>,
    pub market: MarketState,
    #[serde(default)]
    pub office_expansion: OfficeExpansionState,
    #[serde(default)]
    pub player_status: PlayerStatus,
    #[serde(default)]
    pub stage: CompanyStageState,
    #[serde(default)]
    pub deliveries: DeliveryState,
    #[serde(default)]
    pub quests: QuestState,
    pub player_pos: [f32; 3],
    pub player_yaw: f32,
    pub door_open: bool,
}

impl SaveGameData {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != SAVE_SCHEMA_VERSION {
            return Err(format!(
                "Versi save tidak didukung ({} != {}).",
                self.version, SAVE_SCHEMA_VERSION
            ));
        }
        if self.clock.day == 0 || self.clock.hour >= 24 || self.clock.minute >= 60 {
            return Err("Data waktu save tidak valid.".to_string());
        }
        if self.company.name.trim().is_empty() {
            return Err("Nama perusahaan pada file save kosong.".to_string());
        }
        if self.market.price_per_month < 5 || self.market.price_per_month > 100 {
            return Err("Harga pasar pada file save di luar batas.".to_string());
        }
        Ok(())
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Gagal buat folder save: {e}"))?;
        }
        let pretty = ron::ser::PrettyConfig::default();
        let encoded = ron::ser::to_string_pretty(self, pretty)
            .map_err(|e| format!("Gagal serialisasi RON: {e}"))?;
        fs::write(path, encoded).map_err(|e| format!("Gagal tulis file save: {e}"))?;
        Ok(())
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err("File save belum ada (tekan F5 untuk menyimpan dulu).".to_string());
        }
        let raw = fs::read_to_string(path).map_err(|e| format!("Gagal baca file save: {e}"))?;
        let data: Self =
            ron::from_str(&raw).map_err(|e| format!("Format file save rusak: {e}"))?;
        data.validate()?;
        Ok(data)
    }
}

pub fn resolve_save_path() -> PathBuf {
    PathBuf::from("saves").join("savegame.ron")
}

fn handle_save_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<SaveGameCommand>,
) {
    if keys.just_pressed(KeyCode::F5) {
        writer.write(SaveGameCommand::Save);
    } else if keys.just_pressed(KeyCode::F9) {
        writer.write(SaveGameCommand::Load);
    }
}

#[derive(SystemParam)]
struct SaveCoreParams<'w> {
    clock: ResMut<'w, GameClock>,
    company: ResMut<'w, CompanyState>,
    finance: ResMut<'w, FinanceState>,
    product: ResMut<'w, ProductState>,
    employees: ResMut<'w, EmployeeState>,
    market: ResMut<'w, MarketState>,
    expansion: ResMut<'w, OfficeExpansionState>,
    colliders: ResMut<'w, OfficeColliders>,
}

#[derive(SystemParam)]
struct SaveLifeParams<'w> {
    stage: ResMut<'w, CompanyStageState>,
    status: ResMut<'w, PlayerStatus>,
    delivery: ResMut<'w, DeliveryState>,
    quests: ResMut<'w, QuestState>,
}

#[allow(clippy::too_many_arguments)]
fn process_save_commands(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut reader: MessageReader<SaveGameCommand>,
    mut core: SaveCoreParams,
    mut life: SaveLifeParams,
    mut player_q: Query<(&mut Transform, &mut PlayerController)>,
    mut door_q: Query<(&mut OfficeDoor, &mut Interactable)>,
    npc_q: Query<Entity, (With<DeveloperNpc>, Without<PurchasedOfficeProp>)>,
    prop_q: Query<Entity, (With<PurchasedOfficeProp>, Without<DeveloperNpc>)>,
    mut notify: MessageWriter<NotifyMessage>,
) {
    let save_path = resolve_save_path();

    for cmd in reader.read() {
        match cmd {
            SaveGameCommand::Save => {
                let (player_pos, player_yaw) = player_q
                    .single()
                    .map(|(tf, ctrl)| (tf.translation.to_array(), ctrl.yaw))
                    .unwrap_or(([0.8, 0.0, 1.1], 0.25));
                let door_open = door_q.iter().next().is_some_and(|(d, _)| d.is_open);

                let payload = SaveGameData {
                    version: SAVE_SCHEMA_VERSION,
                    clock: SavedClock {
                        day: core.clock.day,
                        hour: core.clock.hour,
                        minute: core.clock.minute,
                    },
                    company: core.company.clone(),
                    finance: core.finance.clone(),
                    product: core.product.clone(),
                    employees: core.employees.roster.clone(),
                    market: core.market.clone(),
                    office_expansion: core.expansion.clone(),
                    player_status: life.status.clone(),
                    stage: life.stage.clone(),
                    deliveries: life.delivery.clone(),
                    quests: life.quests.clone(),
                    player_pos,
                    player_yaw,
                    door_open,
                };

                match payload.save_to_path(&save_path) {
                    Ok(()) => {
                        notify.write(NotifyMessage::info(format!(
                            "Game disimpan (Hari {} · {})",
                            core.clock.day,
                            core.clock.formatted_time()
                        )));
                    }
                    Err(err) => {
                        notify.write(NotifyMessage::warn(err));
                    }
                }
            }
            SaveGameCommand::Load => match SaveGameData::load_from_path(&save_path) {
                Ok(data) => {
                    core.clock.restore(data.clock.day, data.clock.hour, data.clock.minute);
                    *core.company = data.company;
                    *core.finance = data.finance;
                    *core.product = data.product;
                    *core.market = data.market;
                    *core.expansion = data.office_expansion;
                    *life.status = data.player_status;
                    *life.stage = data.stage;
                    *life.delivery = data.deliveries;
                    *life.quests = data.quests;

                    core.employees.max_employees =
                        if core.expansion.is_purchased(OfficeUpgradeKind::ExtraWorkstation)
                            || life.stage.current != crate::life::CompanyStage::Founder
                        {
                            2
                        } else {
                            1
                        };

                    if data.employees.len() <= core.employees.max_employees {
                        core.employees.roster = data.employees;
                    } else {
                        core.employees.roster =
                            data.employees[..core.employees.max_employees].to_vec();
                    }
                    for emp in &mut core.employees.roster {
                        ensure_employee_stats_sane(emp);
                    }
                    core.company.team_size = 1 + core.employees.roster.len() as u32;

                    if let Ok((mut tf, mut ctrl)) = player_q.single_mut() {
                        tf.translation = Vec3::from_array(data.player_pos);
                        ctrl.yaw = data.player_yaw;
                        ctrl.facing_yaw = data.player_yaw;
                        tf.rotation = Quat::from_rotation_y(data.player_yaw);
                    }

                    for (mut door, mut inter) in &mut door_q {
                        door.is_open = data.door_open;
                        door.target_angle = if door.is_open { -1.45 } else { 0.0 };
                        inter.prompt = if door.is_open {
                            "Tutup Pintu".into()
                        } else {
                            "Buka Pintu".into()
                        };
                    }

                    for prop_entity in &prop_q {
                        commands.entity(prop_entity).despawn();
                    }
                    for &kind in &core.expansion.purchased {
                        spawn_upgrade_visual(&mut commands, &asset_server, kind);
                    }
                    rebuild_colliders(&mut core.colliders, &core.expansion);

                    for npc_entity in &npc_q {
                        commands.entity(npc_entity).despawn();
                    }
                    for emp in &core.employees.roster {
                        let seated = emp.state != EmployeeWorkState::WalkingToDesk;
                        spawn_developer_npc(
                            &mut commands,
                            &asset_server,
                            emp.id,
                            emp.desk_index,
                            seated,
                        );
                    }

                    notify.write(NotifyMessage::info(format!(
                        "Save dimuat: Hari {} · {} (${})",
                        core.clock.day,
                        core.clock.formatted_time(),
                        core.finance.cash_balance
                    )));
                }
                Err(err) => {
                    notify.write(NotifyMessage::warn(err));
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::employee::EmployeeFocus;

    fn sample_save() -> SaveGameData {
        let mut comp = CompanyState::default();
        comp.name = "NusaTech Labs".to_string();
        let mut fin = FinanceState::default();
        fin.cash_balance = 19_450;
        SaveGameData {
            version: SAVE_SCHEMA_VERSION,
            clock: SavedClock {
                day: 4,
                hour: 14,
                minute: 30,
            },
            company: comp,
            finance: fin,
            product: ProductState::default(),
            employees: vec![EmployeeProfile {
                id: 1,
                name: "Raka Pratama".to_string(),
                role: "Pembuat Produk".to_string(),
                skill: 1.2,
                daily_salary: 160,
                portrait_path: "characters/employees/portraits/developer_01.png".to_string(),
                state: EmployeeWorkState::Working,
                desk_index: 0,
                energy: 90.0,
                mood: 85.0,
                focus: EmployeeFocus::Balanced,
            }],
            market: MarketState::default(),
            office_expansion: OfficeExpansionState::default(),
            player_status: PlayerStatus::default(),
            stage: CompanyStageState::default(),
            deliveries: DeliveryState::default(),
            quests: QuestState::default(),
            player_pos: [0.5, 0.0, 1.0],
            player_yaw: 0.4,
            door_open: true,
        }
    }

    #[test]
    fn ron_file_roundtrip_preserves_state() {
        let temp_path = std::env::temp_dir().join("startup_tycon_test_save.ron");
        let original = sample_save();
        original.save_to_path(&temp_path).unwrap();

        let loaded = SaveGameData::load_from_path(&temp_path).unwrap();
        let _ = fs::remove_file(&temp_path);

        assert_eq!(loaded.version, SAVE_SCHEMA_VERSION);
        assert_eq!(loaded.clock.day, 4);
        assert_eq!(loaded.clock.hour, 14);
        assert_eq!(loaded.company.name, "NusaTech Labs");
        assert_eq!(loaded.finance.cash_balance, 19_450);
        assert_eq!(loaded.employees.len(), 1);
        assert!(loaded.door_open);
    }

    #[test]
    fn validation_rejects_bad_version_and_invalid_hour() {
        let mut bad_ver = sample_save();
        bad_ver.version = 99;
        assert!(bad_ver.validate().is_err());

        let mut bad_time = sample_save();
        bad_time.clock.hour = 25;
        assert!(bad_time.validate().is_err());
    }
}
