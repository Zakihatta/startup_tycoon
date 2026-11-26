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
