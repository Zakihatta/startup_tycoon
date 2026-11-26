use bevy::prelude::*;

use crate::core::{DayPassedMessage, NotifyMessage};

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FinanceState>()
            .add_systems(Update, process_daily_economy);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TransactionKind {
    InitialCapital,
    Rent,
    Salary,
    ProductCost,
    Revenue,
}

impl TransactionKind {
    pub fn tag(self) -> &'static str {
        match self {
            TransactionKind::InitialCapital => "MODAL",
            TransactionKind::Rent => "SEWA",
            TransactionKind::Salary => "GAJI",
            TransactionKind::ProductCost => "PRODUK",
            TransactionKind::Revenue => "OMZET",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Transaction {
    pub day: u32,
    pub label: String,
    pub amount: i64,
    pub kind: TransactionKind,
}

#[derive(Resource, Debug)]
pub struct FinanceState {
    pub cash_balance: i64,
    pub daily_rent: i64,
    pub total_revenue: i64,
    pub total_expenses: i64,
    pub low_cash_threshold: i64,
    pub transactions: Vec<Transaction>,
}

impl Default for FinanceState {
    fn default() -> Self {
        Self {
            cash_balance: 25_000,
            daily_rent: 150,
            total_revenue: 0,
            total_expenses: 0,
            low_cash_threshold: 5_000,
            transactions: vec![Transaction {
                day: 1,
                label: "Modal Awal Founder".to_string(),
                amount: 25_000,
                kind: TransactionKind::InitialCapital,
            }],
        }
    }
}

impl FinanceState {
    /// Single authoritative entry point for all cash changes.
    pub fn record_transaction(
        &mut self,
        day: u32,
        label: impl Into<String>,
        amount: i64,
        kind: TransactionKind,
    ) {
        self.cash_balance += amount;
        if amount >= 0 && kind != TransactionKind::InitialCapital {
            self.total_revenue += amount;
        } else if amount < 0 {
            self.total_expenses += -amount;
        }
        self.transactions.push(Transaction {
            day,
            label: label.into(),
            amount,
            kind,
        });
    }

    pub fn net_profit(&self) -> i64 {
        self.total_revenue - self.total_expenses
    }

    pub fn is_low_cash(&self) -> bool {
        self.cash_balance < self.low_cash_threshold
    }
}

fn process_daily_economy(
    mut day_reader: MessageReader<DayPassedMessage>,
    mut finance: ResMut<FinanceState>,
    mut notify: MessageWriter<NotifyMessage>,
) {
    for ev in day_reader.read() {
        let rent = finance.daily_rent;
        finance.record_transaction(
            ev.new_day,
            format!("Sewa Kantor (Hari {})", ev.new_day),
            -rent,
            TransactionKind::Rent,
        );
        if finance.is_low_cash() {
            notify.write(NotifyMessage::warn(format!(
                "Hari {}: Kas menipis (${})!",
                ev.new_day, finance.cash_balance
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_cash_and_transactions_work() {
        let mut fin = FinanceState::default();
        assert_eq!(fin.cash_balance, 25_000);
        assert!(!fin.is_low_cash());

        fin.record_transaction(2, "Sewa Kantor (Hari 2)", -150, TransactionKind::Rent);
        fin.record_transaction(2, "Penjualan Awal", 400, TransactionKind::Revenue);

        assert_eq!(fin.cash_balance, 25_250);
        assert_eq!(fin.total_expenses, 150);
        assert_eq!(fin.total_revenue, 400);
        assert_eq!(fin.net_profit(), 250);
        assert_eq!(fin.transactions.len(), 3);
    }

    #[test]
    fn low_cash_warning_triggers_below_threshold() {
        let mut fin = FinanceState::default();
        fin.record_transaction(3, "Pengeluaran Besar", -21_000, TransactionKind::ProductCost);
        assert_eq!(fin.cash_balance, 4_000);
        assert!(fin.is_low_cash());
    }
}
