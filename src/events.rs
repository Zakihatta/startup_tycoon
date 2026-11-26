use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    company::CompanyState,
    core::{DayPassedMessage, GameClock, NotifyMessage},
    economy::{FinanceState, TransactionKind},
    life::{CompanyStage, CompanyStageState, PlayerStatus},
    market::MarketState,
    product::ProductState,
};

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompetitorState>()
            .init_resource::<EventDirectorState>()
            .add_message::<EventDecisionCommand>()
            .add_systems(
                Update,
                (advance_competitors_and_events, handle_event_decision).chain(),
            );
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompetitorCompany {
    pub name: String,
    pub users: u32,
    pub quality: f32,
    pub growth_rate: f32,
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct CompetitorState {
    pub rivals: Vec<CompetitorCompany>,
}

impl Default for CompetitorState {
    fn default() -> Self {
        Self {
            rivals: vec![
                CompetitorCompany {
                    name: "KaryaMuda Studio".to_string(),
                    users: 45,
                    quality: 62.0,
                    growth_rate: 1.05,
                },
                CompetitorCompany {
                    name: "WarungDigital Nusantara".to_string(),
                    users: 110,
                    quality: 74.0,
                    growth_rate: 1.12,
                },
            ],
        }
    }
}

impl CompetitorState {
    pub fn simulate_day(&mut self, day: u32) {
        for (i, rival) in self.rivals.iter_mut().enumerate() {
            let wave = 1.0 + 0.15 * ((day as f32 + i as f32 * 2.0) * 0.4).sin();
            let gain = ((rival.quality * 0.16) * rival.growth_rate * wave).max(2.0).round() as u32;
            rival.users = rival.users.saturating_add(gain);
            rival.quality = (rival.quality + 0.15).min(96.0);
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn leader_users(&self) -> u32 {
        self.rivals.iter().map(|r| r.users).max().unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EverydayEventKind {
    ViralWordOfMouth,
    CustomerComplaint,
    LocalPartnership,
    InvestorOffer,
    CompetitorLaunch,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingEvent {
    pub kind: EverydayEventKind,
    pub title: String,
    pub description: String,
    pub option_a_label: String,
    pub option_b_label: String,
}

impl EverydayEventKind {
    pub fn into_pending(self) -> PendingEvent {
        match self {
            EverydayEventKind::ViralWordOfMouth => PendingEvent {
                kind: self,
                title: "Usaha Kamu Ramai Dibicarakan!".into(),
                description: "Banyak orang mulai membicarakan produkmu dari mulut ke mulut.\nPilih langkah terbaik:".into(),
                option_a_label: "Tambah Layanan & Promo (-$300 | +22 Pelanggan & +4 Nama Baik)".into(),
                option_b_label: "Biarkan Alami ($0 | +10 Pelanggan)".into(),
            },
            EverydayEventKind::CustomerComplaint => PendingEvent {
                kind: self,
                title: "Ada Keluhan dari Beberapa Pelanggan".into(),
                description: "Sebagian pelanggan mengeluh layanan agak lambat hari ini.\nBagaimana reaksimu?".into(),
                option_a_label: "Minta Maaf & Beri Voucher (-$450 | Nama Baik +3 & Kualitas +4)".into(),
                option_b_label: "Lembur Perbaiki Sendiri (-22 Energi | Kualitas +5, Hemat Uang)".into(),
            },
            EverydayEventKind::LocalPartnership => PendingEvent {
                kind: self,
                title: "Ajakan Kerjasama Komunitas Lokal".into(),
                description: "Komunitas UMKM mengajak usahamu ikut pameran akhir pekan ini.".into(),
                option_a_label: "Ikut Pameran (-$600 | +30 Pelanggan & +$1,100 Penjualan)".into(),
                option_b_label: "Fokus di Kantor Saja (+8 Semangat Founder)".into(),
            },
            EverydayEventKind::InvestorOffer => PendingEvent {
                kind: self,
                title: "Penawaran Modal dari Investor!".into(),
                description: "Seorang investor tertarik melihat pertumbuhan usahamu dan menawarkan modal ekspansi.".into(),
                option_a_label: "Terima Investasi (+$8,000 Kas | Bantu Naik Stage & Rekrut Tim)".into(),
                option_b_label: "Tolak & Tetap Mandiri (+8 Nama Baik & +12 Semangat)".into(),
            },
            EverydayEventKind::CompetitorLaunch => PendingEvent {
                kind: self,
                title: "Pesaing Baru Rilis Promo Besar!".into(),
                description: "Perusahaan pesaing meluncurkan promo murah untuk menarik pelangganmu.".into(),
                option_a_label: "Balas dengan Kampanye Promosi (-$500 | +20 Pelanggan & Nama Baik +2)".into(),
                option_b_label: "Perkuat Kualitas Layanan (-$350 | Kualitas Produk +5)".into(),
            },
        }
    }
}

#[derive(Resource, Clone, Default, Debug, Serialize, Deserialize)]
pub struct EventDirectorState {
    pub active_event: Option<PendingEvent>,
    pub last_event_day: u32,
    pub investor_accepted_count: u32,
    pub events_resolved: u32,
}

#[derive(Message, Clone, Copy, Debug)]
pub enum EventDecisionCommand {
    ChooseOptionA,
    ChooseOptionB,
}

fn advance_competitors_and_events(
    mut day_reader: MessageReader<DayPassedMessage>,
    stage: Res<CompanyStageState>,
    product: Res<ProductState>,
    mut competitors: ResMut<CompetitorState>,
    mut director: ResMut<EventDirectorState>,
    mut notify: MessageWriter<NotifyMessage>,
) {
    let has_launched = product
        .active
        .as_ref()
        .is_some_and(|p| matches!(p.phase, crate::product::ProductPhase::Launched))
        || !product.launched_portfolio.is_empty();

    for ev in day_reader.read() {
        competitors.simulate_day(ev.new_day);

        if director.active_event.is_some() || !has_launched || ev.new_day < 3 {
            continue;
        }
        if ev.new_day.saturating_sub(director.last_event_day) < 3 {
            continue;
        }

        let pick = match (stage.current, (ev.new_day % 4) as usize) {
            (CompanyStage::Founder, 0) | (CompanyStage::Founder, 2) => {
                EverydayEventKind::ViralWordOfMouth
            }
            (CompanyStage::Founder, 1) => EverydayEventKind::CustomerComplaint,
            (CompanyStage::Founder, _) => EverydayEventKind::LocalPartnership,
            (CompanyStage::SmallStartup, 0) => EverydayEventKind::InvestorOffer,
            (CompanyStage::SmallStartup, 1) => EverydayEventKind::CustomerComplaint,
            (CompanyStage::SmallStartup, 2) => EverydayEventKind::LocalPartnership,
            (CompanyStage::SmallStartup, _) => EverydayEventKind::ViralWordOfMouth,
            (_, 0) => EverydayEventKind::CompetitorLaunch,
            (_, 1) => EverydayEventKind::InvestorOffer,
            (_, 2) => EverydayEventKind::CustomerComplaint,
            (_, _) => EverydayEventKind::LocalPartnership,
        };

        let pending = pick.into_pending();
        director.last_event_day = ev.new_day;
        notify.write(NotifyMessage::info(format!(
            "Kejadian Usaha: {}",
            pending.title
        )));
        director.active_event = Some(pending);
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_event_decision(
    mut reader: MessageReader<EventDecisionCommand>,
    clock: Res<GameClock>,
    mut director: ResMut<EventDirectorState>,
    mut finance: ResMut<FinanceState>,
    mut market: ResMut<MarketState>,
    mut company: ResMut<CompanyState>,
    mut product: ResMut<ProductState>,
    mut status: ResMut<PlayerStatus>,
    mut notify: MessageWriter<NotifyMessage>,
) {
    for cmd in reader.read() {
        let Some(event) = director.active_event.take() else {
            continue;
        };

        let choose_a = matches!(cmd, EventDecisionCommand::ChooseOptionA);
        match event.kind {
            EverydayEventKind::ViralWordOfMouth => {
                if choose_a {
                    if finance.cash_balance >= 300 {
                        finance.record_transaction(
                            clock.day,
                            "Promo Viral Produk",
                            -300,
                            TransactionKind::ProductCost,
                        );
                        market.active_users += 22;
                        company.reputation = (company.reputation + 4.0).min(100.0);
                        notify.write(NotifyMessage::info(
                            "Promo berhasil! +22 pelanggan baru & Nama Baik naik.",
                        ));
                    } else {
                        market.active_users += 10;
                        notify.write(NotifyMessage::warn(
                            "Kas kurang dari $300, promosi berjalan alami (+10 pelanggan).",
                        ));
                    }
                } else {
                    market.active_users += 10;
                    notify.write(NotifyMessage::info(
                        "Kabar baik menyebar alami (+10 pelanggan baru).",
                    ));
                }
            }
            EverydayEventKind::CustomerComplaint => {
                if choose_a {
                    if finance.cash_balance >= 450 {
                        finance.record_transaction(
                            clock.day,
                            "Kompensasi Pelanggan",
                            -450,
                            TransactionKind::ProductCost,
                        );
                        company.reputation = (company.reputation + 3.0).min(100.0);
                        if let Some(p) = &mut product.active {
                            p.quality = (p.quality + 4.0).min(100.0);
                        }
                        notify.write(NotifyMessage::info(
                            "Pelanggan puas dengan respon cepatmu! Kualitas & Nama Baik naik.",
                        ));
                    } else {
                        status.energy = (status.energy - 15.0).max(5.0);
                        notify.write(NotifyMessage::warn(
                            "Kas kurang — Founder turun tangan langsung memperbaiki masalah.",
                        ));
                    }
                } else {
                    status.energy = (status.energy - 22.0).max(5.0);
                    if let Some(p) = &mut product.active {
                        p.quality = (p.quality + 5.0).min(100.0);
                    }
                    notify.write(NotifyMessage::info(
                        "Lembur perbaikan selesai! Kualitas +5 (Energi berkurang).",
                    ));
                }
            }
            EverydayEventKind::LocalPartnership => {
                if choose_a {
                    if finance.cash_balance >= 600 {
                        finance.record_transaction(
                            clock.day,
                            "Biaya Pameran Lokal",
                            -600,
                            TransactionKind::ProductCost,
                        );
                        finance.record_transaction(
                            clock.day,
                            "Hasil Penjualan Pameran",
                            1_100,
                            TransactionKind::Revenue,
                        );
                        market.active_users += 30;
                        notify.write(NotifyMessage::info(
                            "Pameran sukses! +30 pelanggan baru & pemasukan +$1,100!",
                        ));
                    } else {
                        status.mood = (status.mood + 6.0).min(100.0);
                        notify.write(NotifyMessage::warn("Kas kurang dari $600 untuk ikut pameran."));
                    }
                } else {
                    status.mood = (status.mood + 8.0).min(100.0);
                    notify.write(NotifyMessage::info("Tetap fokus bekerja dengan tenang (+8 Semangat)."));
                }
            }
            EverydayEventKind::InvestorOffer => {
                if choose_a {
                    finance.record_transaction(
                        clock.day,
                        "Dana Investasi Masuk",
                        8_000,
                        TransactionKind::InitialCapital,
                    );
                    director.investor_accepted_count += 1;
                    notify.write(NotifyMessage::info(
                        "Investasi +$8,000 diterima! Gunakan untuk ekspansi & tim.",
                    ));
                } else {
                    company.reputation = (company.reputation + 8.0).min(100.0);
                    status.mood = (status.mood + 12.0).min(100.0);
                    notify.write(NotifyMessage::info(
                        "Usaha tetap mandiri! Nama Baik +8 & Semangat +12.",
                    ));
                }
            }
            EverydayEventKind::CompetitorLaunch => {
                if choose_a {
                    if finance.cash_balance >= 500 {
                        finance.record_transaction(
                            clock.day,
                            "Promo Tanding Pesaing",
                            -500,
                            TransactionKind::ProductCost,
                        );
                        market.active_users += 20;
                        company.reputation = (company.reputation + 2.0).min(100.0);
                        notify.write(NotifyMessage::info(
                            "Promo tanding berhasil menarik +20 pelanggan baru!",
                        ));
                    } else {
                        notify.write(NotifyMessage::warn("Kas kurang untuk promo tanding."));
                    }
                } else if finance.cash_balance >= 350 {
                    finance.record_transaction(
                        clock.day,
                        "Peningkatan Mutu Layanan",
                        -350,
                        TransactionKind::ProductCost,
                    );
                    if let Some(p) = &mut product.active {
                        p.quality = (p.quality + 5.0).min(100.0);
                    }
                    notify.write(NotifyMessage::info("Kualitas produk ditingkatkan (+5)!"));
                } else {
                    notify.write(NotifyMessage::warn("Kas kurang, tetap pertahankan layanan harian."));
                }
            }
        }

        director.events_resolved += 1;
        let leveled = status.gain_xp(35);
        if leveled {
            notify.write(NotifyMessage::info(format!(
                "Level Founder naik ke Level {}!",
                status.level
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn competitors_grow_steadily_over_days() {
        let mut comp = CompetitorState::default();
        let before = comp.leader_users();
        comp.simulate_day(2);
        comp.simulate_day(3);
        assert!(comp.leader_users() > before);
    }
}
