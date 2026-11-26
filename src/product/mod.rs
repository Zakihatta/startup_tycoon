use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    company::CompanyState,
    core::{DayPassedMessage, GameClock, HourPassedMessage, NotifyMessage},
    economy::{FinanceState, TransactionKind},
    life::PlayerStatus,
};

pub struct ProductPlugin;

impl Plugin for ProductPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProductState>()
            .add_message::<ProductCommand>()
            .add_systems(
                Update,
                (
                    handle_product_commands,
                    advance_product_hourly,
                    charge_product_daily_cost,
                ),
            );
    }
}

pub const PRODUCT_START_COST: i64 = 1_500;
pub const PRODUCT_DAILY_DEV_COST: i64 = 100;
pub const PRODUCT_POLISH_COST: i64 = 800;

fn default_version() -> u32 {
    1
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductPhase {
    Development,
    Testing,
    ReadyToLaunch,
    Launched,
}

impl ProductPhase {
    pub fn label(self) -> &'static str {
        match self {
            ProductPhase::Development => "Proses Pembuatan",
            ProductPhase::Testing => "Cek Kualitas",
            ProductPhase::ReadyToLaunch => "Siap Diluncurkan",
            ProductPhase::Launched => "Sudah Dipasarkan",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoftwareProduct {
    pub name: String,
    pub category: String,
    pub phase: ProductPhase,
    pub progress: f32,
    pub quality: f32,
    pub start_day: u32,
    pub launch_day: Option<u32>,
    #[serde(default = "default_version")]
    pub version: u32,
}

#[derive(Resource, Clone, Default, Debug, Serialize, Deserialize)]
pub struct ProductState {
    pub active: Option<SoftwareProduct>,
    pub total_productivity_bonus: f32,
    #[serde(default)]
    pub launched_portfolio: Vec<SoftwareProduct>,
}

#[derive(Message, Clone, Debug)]
pub enum ProductCommand {
    StartProject { name: String, category: String },
    LaunchProduct,
    PolishActiveProduct,
    StartNextProject { name: String, category: String },
}

impl ProductState {
    pub fn start_project(
        &mut self,
        name: impl Into<String>,
        category: impl Into<String>,
        day: u32,
        finance: &mut FinanceState,
    ) -> Result<(), &'static str> {
        if self.active.is_some() {
            return Err("Proyek produk sudah ada.");
        }
        if finance.cash_balance < PRODUCT_START_COST {
            return Err("Saldo kas tidak mencukupi ($1,500).");
        }
        let name_s = name.into();
        finance.record_transaction(
            day,
            format!("Inisiasi Produk: {name_s}"),
            -PRODUCT_START_COST,
            TransactionKind::ProductCost,
        );
        self.active = Some(SoftwareProduct {
            name: name_s,
            category: category.into(),
            phase: ProductPhase::Development,
            progress: 0.0,
            quality: 20.0,
            start_day: day,
            launch_day: None,
            version: 1,
        });
        Ok(())
    }

    pub fn launched_count(&self) -> usize {
        let active_launched = usize::from(
            self.active
                .as_ref()
                .is_some_and(|p| p.phase == ProductPhase::Launched),
        );
        self.launched_portfolio.len() + active_launched
    }

    pub fn effective_market_quality(&self) -> f32 {
        let active_q = self
            .active
            .as_ref()
            .filter(|p| p.phase == ProductPhase::Launched)
            .map_or(0.0, |p| p.quality);
        let portfolio_q: f32 = self
            .launched_portfolio
            .iter()
            .map(|p| p.quality * 0.45)
            .sum();
        active_q + portfolio_q
    }

    pub fn polish_active(
        &mut self,
        day: u32,
        finance: &mut FinanceState,
    ) -> Result<u32, &'static str> {
        let Some(prod) = &mut self.active else {
            return Err("Belum ada produk aktif.");
        };
        if prod.phase != ProductPhase::Launched {
            return Err("Luncurkan produk dulu sebelum membuat versi pembaruan.");
        }
        if finance.cash_balance < PRODUCT_POLISH_COST {
            return Err("Saldo kas kurang ($800) untuk pembaruan kualitas.");
        }
        finance.record_transaction(
            day,
            format!("Update {} v{}", prod.name, prod.version + 1),
            -PRODUCT_POLISH_COST,
            TransactionKind::ProductCost,
        );
        prod.version += 1;
        prod.quality = (prod.quality + 8.0).min(100.0);
        Ok(prod.version)
    }

    pub fn start_next_project(
        &mut self,
        name: impl Into<String>,
        category: impl Into<String>,
        day: u32,
        finance: &mut FinanceState,
    ) -> Result<(), &'static str> {
        if let Some(cur) = self.active.as_ref() {
            if cur.phase != ProductPhase::Launched {
                return Err("Selesaikan & luncurkan produk yang sedang berjalan dulu.");
            }
        }
        if finance.cash_balance < PRODUCT_START_COST {
            return Err("Saldo kas tidak mencukupi ($1,500) untuk produk baru.");
        }
        if let Some(old) = self.active.take() {
            self.launched_portfolio.push(old);
        }
        self.start_project(name, category, day, finance)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn advance_hour(&mut self, founder_active: bool) {
        self.advance_hour_with_efficiency(founder_active, 1.0);
    }

    pub fn advance_hour_with_efficiency(&mut self, founder_active: bool, founder_mult: f32) {
        let Some(prod) = &mut self.active else {
            return;
        };

        // Base founder output scaled by energy/hunger/level + employee bonus
        let founder_rate = if founder_active { 2.2 * founder_mult } else { 0.0 };
        let rate = founder_rate + self.total_productivity_bonus;

        match prod.phase {
            ProductPhase::Development => {
                prod.progress = (prod.progress + rate).min(100.0);
                prod.quality = (prod.quality + rate * 0.45).min(85.0);
                if prod.progress >= 100.0 {
                    prod.phase = ProductPhase::Testing;
                    prod.progress = 0.0;
                }
            }
            ProductPhase::Testing => {
                prod.progress = (prod.progress + rate * 1.35).min(100.0);
                prod.quality = (prod.quality + rate * 0.55).min(100.0);
                if prod.progress >= 100.0 {
                    prod.phase = ProductPhase::ReadyToLaunch;
                    prod.progress = 100.0;
                }
            }
            ProductPhase::ReadyToLaunch | ProductPhase::Launched => {}
        }
    }

    pub fn launch(&mut self, day: u32, company: &mut CompanyState) -> bool {
        let Some(prod) = &mut self.active else {
            return false;
        };
        if prod.phase != ProductPhase::ReadyToLaunch {
            return false;
        }
        prod.phase = ProductPhase::Launched;
        prod.launch_day = Some(day);
        company.reputation += 5.0 + (prod.quality * 0.05);
        true
    }
}

fn handle_product_commands(
    mut reader: MessageReader<ProductCommand>,
    clock: Res<GameClock>,
    mut product: ResMut<ProductState>,
    mut finance: ResMut<FinanceState>,
    mut company: ResMut<CompanyState>,
    mut notify: MessageWriter<NotifyMessage>,
) {
    for cmd in reader.read() {
        match cmd {
            ProductCommand::StartProject { name, category } => {
                match product.start_project(name.clone(), category.clone(), clock.day, &mut finance) {
                    Ok(()) => {
                        notify.write(NotifyMessage::info(format!(
                            "Proyek '{name}' dimulai!"
                        )));
                    }
                    Err(err) => {
                        notify.write(NotifyMessage::warn(err));
                    }
                }
            }
            ProductCommand::LaunchProduct => {
                if product.launch(clock.day, &mut company) {
                    if let Some(p) = &product.active {
                        notify.write(NotifyMessage::info(format!(
                            "Produk '{}' resmi dirilis!",
                            p.name
                        )));
                    }
                }
            }
            ProductCommand::PolishActiveProduct => {
                match product.polish_active(clock.day, &mut finance) {
                    Ok(ver) => {
                        company.reputation = (company.reputation + 2.5).min(100.0);
                        notify.write(NotifyMessage::info(format!(
                            "Pembaruan v{ver} selesai! Kualitas & Nama Baik meningkat."
                        )));
                    }
                    Err(err) => {
                        notify.write(NotifyMessage::warn(err));
                    }
                }
            }
            ProductCommand::StartNextProject { name, category } => {
                match product.start_next_project(name.clone(), category.clone(), clock.day, &mut finance) {
                    Ok(()) => {
                        notify.write(NotifyMessage::info(format!(
                            "Produk baru '{name}' dimulai! Produk sebelumnya tetap menghasilkan pendapatan."
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

fn advance_product_hourly(
    mut hour_reader: MessageReader<HourPassedMessage>,
    status: Res<PlayerStatus>,
    mut product: ResMut<ProductState>,
    mut notify: MessageWriter<NotifyMessage>,
) {
    let mult = status.work_efficiency();
    for ev in hour_reader.read() {
        // Work hours: 08:00 - 21:00
        let working = ev.hour >= 8 && ev.hour <= 21;
        if working {
            let prev_phase = product.active.as_ref().map(|p| p.phase);
            product.advance_hour_with_efficiency(true, mult);
            let new_phase = product.active.as_ref().map(|p| p.phase);
            if prev_phase != new_phase
                && let Some(p) = &product.active
            {
                match p.phase {
                    ProductPhase::Testing => {
                        notify.write(NotifyMessage::info(format!(
                            "'{}' masuk tahap Cek Kualitas!",
                            p.name
                        )));
                    }
                    ProductPhase::ReadyToLaunch => {
                        notify.write(NotifyMessage::info(format!(
                            "'{}' SIAP DILUNCURKAN! Buka Kelola Usaha -> Produk.",
                            p.name
                        )));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn charge_product_daily_cost(
    mut day_reader: MessageReader<DayPassedMessage>,
    product: Res<ProductState>,
    mut finance: ResMut<FinanceState>,
) {
    for ev in day_reader.read() {
        let is_building = product.active.as_ref().is_some_and(|p| {
            matches!(p.phase, ProductPhase::Development | ProductPhase::Testing)
        });
        if is_building {
            finance.record_transaction(
                ev.new_day,
                format!("Biaya R&D Produk (Hari {})", ev.new_day),
                -PRODUCT_DAILY_DEV_COST,
                TransactionKind::ProductCost,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_project_deducts_cost_and_enters_development() {
        let mut prod = ProductState::default();
        let mut fin = FinanceState::default();
        assert!(prod.start_project("TaskFlow SaaS", "Productivity", 1, &mut fin).is_ok());
        assert_eq!(fin.cash_balance, 25_000 - PRODUCT_START_COST);
        let p = prod.active.as_ref().unwrap();
        assert_eq!(p.phase, ProductPhase::Development);
        assert_eq!(p.progress, 0.0);
    }

    #[test]
    fn product_progresses_through_development_testing_and_launch() {
        let mut prod = ProductState::default();
        let mut fin = FinanceState::default();
        let mut comp = CompanyState::default();

        prod.start_project("TaskFlow SaaS", "Productivity", 1, &mut fin).unwrap();

        for _ in 0..50 {
            prod.advance_hour(true);
        }
        assert_eq!(prod.active.as_ref().unwrap().phase, ProductPhase::Testing);

        for _ in 0..40 {
            prod.advance_hour(true);
        }
        assert_eq!(prod.active.as_ref().unwrap().phase, ProductPhase::ReadyToLaunch);

        assert!(prod.launch(5, &mut comp));
        let p = prod.active.as_ref().unwrap();
        assert_eq!(p.phase, ProductPhase::Launched);
        assert_eq!(p.launch_day, Some(5));
        assert!(comp.reputation > 10.0);
    }
}
