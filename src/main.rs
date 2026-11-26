use bevy::{asset::AssetPlugin, prelude::*};
use std::path::PathBuf;

mod company;
mod core;
mod economy;
mod employee;
mod interaction;
mod market;
mod office;
mod player;
mod product;
mod save_system;
mod ui;

use company::CompanyPlugin;
use core::{NotificationPlugin, TimePlugin};
use economy::EconomyPlugin;
use employee::EmployeePlugin;
use interaction::InteractionPlugin;
use market::MarketPlugin;
use office::OfficePlugin;
use player::PlayerPlugin;
use product::ProductPlugin;
use save_system::SaveSystemPlugin;
use ui::GameUiPlugin;

fn resolve_assets_dir() -> String {
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let beside_exe = exe_dir.join("assets");
        if beside_exe.is_dir() {
            return beside_exe.to_string_lossy().into_owned();
        }
    }
    let cwd_assets = PathBuf::from("assets");
    if cwd_assets.is_dir()
        && let Ok(abs) = std::fs::canonicalize(&cwd_assets)
    {
        return abs.to_string_lossy().into_owned();
    }
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        let manifest_assets = PathBuf::from(manifest_dir).join("assets");
        if manifest_assets.is_dir() {
            return manifest_assets.to_string_lossy().into_owned();
        }
    }
    "assets".to_string()
}

fn main() {
    let assets_dir = resolve_assets_dir();
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.68, 0.80, 0.92)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.98, 0.96, 0.92),
            brightness: 450.0,
            ..default()
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: assets_dir,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Startup Tycoon - MVP Prototype".into(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            TimePlugin,
            NotificationPlugin,
            CompanyPlugin,
            EconomyPlugin,
            ProductPlugin,
            EmployeePlugin,
            MarketPlugin,
            OfficePlugin,
            PlayerPlugin,
            InteractionPlugin,
            SaveSystemPlugin,
            GameUiPlugin,
        ))
        .run();

    // Ensure background audio/gamepad threads exit cleanly when the window closes
    std::process::exit(0);
}

#[cfg(test)]
mod mvp_integration_tests {
    use crate::{
        company::CompanyState,
        economy::{FinanceState, TransactionKind},
        employee::{EmployeeState, EmployeeWorkState},
        market::MarketState,
        product::{ProductPhase, ProductState, PRODUCT_DAILY_DEV_COST},
    };

    #[test]
    fn mvp_full_loop_reaches_profitability() {
        let mut company = CompanyState::default();
        let mut finance = FinanceState::default();
        let mut employees = EmployeeState::default();
        let mut product = ProductState::default();
        let mut market = MarketState::default();

        // Day 1: Hire Developer #0 (Raka, $600 fee, $160/day) & start product ($1,500)
        employees.hire_candidate(0, 1, &mut finance, &mut company).unwrap();
        employees.roster[0].state = EmployeeWorkState::Working;
        product.total_productivity_bonus = employees.productivity_bonus();
        product
            .start_project("TaskFlow SaaS", "Productivity", 1, &mut finance)
            .unwrap();

        // Simulate working days until product is ReadyToLaunch
        let mut day = 1u32;
        while product.active.as_ref().unwrap().phase != ProductPhase::ReadyToLaunch {
            for _hour in 8..=21 {
                product.advance_hour(true);
            }
            day += 1;
            finance.record_transaction(day, "Sewa", -finance.daily_rent, TransactionKind::Rent);
            finance.record_transaction(
                day,
                "Gaji",
                -employees.roster[0].daily_salary,
                TransactionKind::Salary,
            );
            if matches!(
                product.active.as_ref().unwrap().phase,
                ProductPhase::Development | ProductPhase::Testing
            ) {
                finance.record_transaction(
                    day,
                    "R&D",
                    -PRODUCT_DAILY_DEV_COST,
                    TransactionKind::ProductCost,
                );
            }
            assert!(day < 20, "Product took too long to finish");
        }

        // Launch product
        assert!(product.launch(day, &mut company));
        let prod_quality = product.active.as_ref().unwrap().quality;

        // Simulate 15 days post-launch
        for _ in 0..15 {
            day += 1;
            finance.record_transaction(day, "Sewa", -finance.daily_rent, TransactionKind::Rent);
            finance.record_transaction(
                day,
                "Gaji",
                -employees.roster[0].daily_salary,
                TransactionKind::Salary,
            );
            let snap = market.simulate_day(day, prod_quality, company.reputation);
            finance.record_transaction(day, "Omzet", snap.revenue, TransactionKind::Revenue);
            assert!(finance.cash_balance > 0, "Ran out of cash on day {day}");
        }

        let daily_burn = finance.daily_rent + employees.roster[0].daily_salary;
        assert!(
            market.last_daily_revenue > daily_burn,
            "Expected profitable daily revenue (got {}, burn {})",
            market.last_daily_revenue,
            daily_burn
        );
    }
}
