use bevy::prelude::*;

use crate::{
    company::CompanyState,
    core::DayPassedMessage,
    economy::{FinanceState, TransactionKind},
    product::{ProductPhase, ProductState},
};

pub struct MarketPlugin;

impl Plugin for MarketPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MarketState>()
            .add_message::<MarketCommand>()
            .add_systems(Update, (handle_market_commands, process_daily_market));
    }
}

#[derive(Clone, Debug)]
pub struct DailySalesSnapshot {
    pub day: u32,
    pub new_users: u32,
    pub churned_users: u32,
    pub active_users: u32,
    pub revenue: i64,
}

#[derive(Resource, Debug)]
pub struct MarketState {
    pub base_demand: f32,
    pub price_per_month: i64,
    pub active_users: u32,
    pub last_daily_revenue: i64,
    pub history: Vec<DailySalesSnapshot>,
}

impl Default for MarketState {
    fn default() -> Self {
        Self {
            base_demand: 1.0,
            price_per_month: 15,
            active_users: 0,
            last_daily_revenue: 0,
            history: Vec::new(),
        }
    }
}

impl MarketState {
    pub fn price_demand_factor(&self) -> f32 {
        match self.price_per_month {
            0..=10 => 1.35,
            11..=18 => 1.00,
            _ => 0.62,
        }
    }

    pub fn churn_rate(&self) -> f32 {
        let extra = ((self.price_per_month - 10).max(0) as f32) * 0.003;
        (0.04 + extra).clamp(0.03, 0.14)
    }

    pub fn demand_label(&self) -> &'static str {
        if self.base_demand >= 1.15 {
            "Tinggi (Tumbuh)"
        } else if self.base_demand <= 0.85 {
            "Lesu"
        } else {
            "Stabil"
        }
    }

    /// Simulates one day of user growth, churn, and subscription revenue.
    pub fn simulate_day(
        &mut self,
        day: u32,
        product_quality: f32,
        company_reputation: f32,
    ) -> DailySalesSnapshot {
        // Gentle deterministic market wave around 1.0 (0.90..1.15)
        self.base_demand = 1.02 + 0.12 * ((day as f32) * 0.45).sin();

        let acquisition_raw = (product_quality * 1.35 + company_reputation * 2.1)
            * self.base_demand
            * self.price_demand_factor();
        let new_users = acquisition_raw.max(0.0).round() as u32;

        let churned_users = ((self.active_users as f32) * self.churn_rate()).round() as u32;
        self.active_users = self
            .active_users
            .saturating_sub(churned_users)
            .saturating_add(new_users);

        let daily_arpu = (self.price_per_month as f32) / 30.0;
        let revenue = ((self.active_users as f32) * daily_arpu).round() as i64;
        self.last_daily_revenue = revenue;

        let snap = DailySalesSnapshot {
            day,
            new_users,
            churned_users,
            active_users: self.active_users,
            revenue,
        };
        self.history.push(snap.clone());
        if self.history.len() > 14 {
            self.history.remove(0);
        }
        snap
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub enum MarketCommand {
    SetPrice(i64),
}

fn handle_market_commands(
    mut reader: MessageReader<MarketCommand>,
    mut market: ResMut<MarketState>,
) {
    for cmd in reader.read() {
        match cmd {
            MarketCommand::SetPrice(price) => {
                market.price_per_month = (*price).clamp(5, 50);
            }
        }
    }
}

fn process_daily_market(
    mut day_reader: MessageReader<DayPassedMessage>,
    product: Res<ProductState>,
    mut company: ResMut<CompanyState>,
    mut market: ResMut<MarketState>,
    mut finance: ResMut<FinanceState>,
) {
    let Some(prod) = &product.active else {
        return;
    };
    if prod.phase != ProductPhase::Launched {
        return;
    }

    for ev in day_reader.read() {
        let snap = market.simulate_day(ev.new_day, prod.quality, company.reputation);
        if snap.revenue > 0 {
            finance.record_transaction(
                ev.new_day,
                format!("Omzet {} ({} usr)", prod.name, snap.active_users),
                snap.revenue,
                TransactionKind::Revenue,
            );
        }
        company.reputation = (company.reputation + 0.15).min(100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launched_product_generates_users_and_revenue() {
        let mut market = MarketState::default();
        let snap1 = market.simulate_day(2, 70.0, 20.0);
        let snap2 = market.simulate_day(3, 70.0, 20.0);

        assert!(snap1.active_users > 0);
        assert!(snap1.revenue > 0);
        assert!(snap2.active_users > snap1.active_users);
    }

    #[test]
    fn lower_price_drives_faster_user_growth() {
        let mut cheap = MarketState {
            price_per_month: 10,
            ..default()
        };
        let mut pricey = MarketState {
            price_per_month: 25,
            ..default()
        };

        let s_cheap = cheap.simulate_day(2, 75.0, 20.0);
        let s_pricey = pricey.simulate_day(2, 75.0, 20.0);
        assert!(s_cheap.new_users > s_pricey.new_users);
    }
}
