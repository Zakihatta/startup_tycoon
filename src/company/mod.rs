use bevy::prelude::*;

pub struct CompanyPlugin;

impl Plugin for CompanyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompanyState>();
    }
}

/// Authoritative company identity & progression (cash lives in FinanceState).
#[derive(Resource, Debug)]
pub struct CompanyState {
    pub name: String,
    pub industry: String,
    pub office_tier: String,
    pub reputation: f32,
    pub team_size: u32,
}

impl CompanyState {
    pub const PRESET_NAMES: [&'static str; 4] = [
        "Startup Kamu",
        "NusaTech Labs",
        "KopiKode Studio",
        "AwanSoft ID",
    ];

    pub fn cycle_name(&mut self) {
        let idx = Self::PRESET_NAMES
            .iter()
            .position(|&n| n == self.name)
            .map_or(0, |i| (i + 1) % Self::PRESET_NAMES.len());
        self.name = Self::PRESET_NAMES[idx].to_string();
    }
}

impl Default for CompanyState {
    fn default() -> Self {
        Self {
            name: "Startup Kamu".to_string(),
            industry: "Software & Teknologi".to_string(),
            office_tier: "Ruang Sewaan Kecil".to_string(),
            reputation: 10.0,
            team_size: 1, // Founder
        }
    }
}
