use bevy::prelude::*;
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
    market::MarketState,
    office::OfficeDoor,
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

#[allow(clippy::too_many_arguments)]
fn process_save_commands(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut reader: MessageReader<SaveGameCommand>,
    mut clock: ResMut<GameClock>,
    mut company: ResMut<CompanyState>,
    mut finance: ResMut<FinanceState>,
    mut product: ResMut<ProductState>,
    mut employees: ResMut<EmployeeState>,
    mut market: ResMut<MarketState>,
    mut player_q: Query<(&mut Transform, &mut PlayerController)>,
    mut door_q: Query<(&mut OfficeDoor, &mut Interactable)>,
    npc_q: Query<Entity, With<DeveloperNpc>>,
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
                        day: clock.day,
                        hour: clock.hour,
                        minute: clock.minute,
                    },
                    company: company.clone(),
                    finance: finance.clone(),
                    product: product.clone(),
                    employees: employees.roster.clone(),
                    market: market.clone(),
                    player_pos,
                    player_yaw,
                    door_open,
                };

                match payload.save_to_path(&save_path) {
                    Ok(()) => {
                        notify.write(NotifyMessage::info(format!(
                            "Game disimpan (Hari {} · {})",
                            clock.day,
                            clock.formatted_time()
                        )));
                    }
                    Err(err) => {
                        notify.write(NotifyMessage::warn(err));
                    }
                }
            }
            SaveGameCommand::Load => match SaveGameData::load_from_path(&save_path) {
                Ok(data) => {
                    clock.restore(data.clock.day, data.clock.hour, data.clock.minute);
                    *company = data.company;
                    *finance = data.finance;
                    *product = data.product;
                    *market = data.market;

                    if data.employees.len() <= employees.max_employees {
                        employees.roster = data.employees;
                    } else {
                        employees.roster = data.employees[..employees.max_employees].to_vec();
                    }
                    company.team_size = 1 + employees.roster.len() as u32;

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

                    for npc_entity in &npc_q {
                        commands.entity(npc_entity).despawn();
                    }
                    for emp in &employees.roster {
                        let seated = emp.state != EmployeeWorkState::WalkingToDesk;
                        spawn_developer_npc(&mut commands, &asset_server, emp.id, seated);
                    }

                    notify.write(NotifyMessage::info(format!(
                        "Save dimuat: Hari {} · {} (${})",
                        clock.day,
                        clock.formatted_time(),
                        finance.cash_balance
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
                role: "Developer".to_string(),
                skill: 1.2,
                daily_salary: 160,
                portrait_path: "characters/employees/portraits/developer_01.png".to_string(),
                state: EmployeeWorkState::Working,
            }],
            market: MarketState::default(),
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
