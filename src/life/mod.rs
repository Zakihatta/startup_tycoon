use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    f32::consts::PI,
    time::Duration,
};

use crate::{
    company::CompanyState,
    core::{GameClock, HourPassedMessage, NotifyMessage},
    economy::{FinanceState, TransactionKind},
    employee::{EmployeeProfile, EmployeeState},
    interaction::{InteractTriggered, Interactable, InteractableKind},
    market::MarketState,
    office::{
        rebuild_colliders, spawn_upgrade_visual, OfficeColliders, OfficeExpansionState,
        OfficeUpgradeKind,
    },
    product::{ProductPhase, ProductState},
};

const COURIER_ASSET: &str = "characters/courier/character_courier.glb";

pub struct LifePlugin;

impl Plugin for LifePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompanyStageState>()
            .init_resource::<PlayerStatus>()
            .init_resource::<DeliveryState>()
            .init_resource::<QuestState>()
            .add_message::<LifeCommand>()
            .add_message::<PickupSoundEvent>()
            .add_systems(Startup, setup_courier_anim_graph)
            .add_systems(
                Update,
                (
                    handle_life_commands,
                    tick_player_needs_hourly,
                    advance_deliveries_and_couriers,
                    setup_courier_animations,
                    sync_courier_animations,
                    handle_courier_and_sofa_interactions,
                    evaluate_quests_and_milestones,
                )
                    .chain(),
            );
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub struct PickupSoundEvent;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompanyStage {
    #[default]
    Founder,
    SmallStartup,
    GrowingCompany,
    Corporation,
}

impl CompanyStage {
    pub fn title(self) -> &'static str {
        match self {
            CompanyStage::Founder => "Stage 1: The Founder",
            CompanyStage::SmallStartup => "Stage 2: Small Startup",
            CompanyStage::GrowingCompany => "Stage 3: Growing Company",
            CompanyStage::Corporation => "Stage 4: Corporation",
        }
    }

    pub fn place_label(self) -> &'static str {
        match self {
            CompanyStage::Founder => "Kamar / Studio Kecil",
            CompanyStage::SmallStartup => "Studio Kantor Berkembang",
            CompanyStage::GrowingCompany => "Kantor Tim & Resepsionis",
            CompanyStage::Corporation => "Gedung Perusahaan",
        }
    }
}

#[derive(Resource, Clone, Default, Debug, Serialize, Deserialize)]
pub struct CompanyStageState {
    pub current: CompanyStage,
}

pub const STAGE_2_UPGRADE_COST: i64 = 3_500;
pub const STAGE_3_UPGRADE_COST: i64 = 8_000;
pub const STAGE_4_UPGRADE_COST: i64 = 18_000;

impl CompanyStageState {
    pub fn can_upgrade_to_stage_2(
        &self,
        status: &PlayerStatus,
        product: &ProductState,
        market: &MarketState,
        finance: &FinanceState,
    ) -> Result<(), &'static str> {
        if self.current != CompanyStage::Founder {
            return Err("Usaha sudah berada di Stage 2 atau lebih tinggi.");
        }
        let has_launched = status.products_launched >= 1
            || product.launched_count() >= 1;
        if !has_launched {
            return Err("Luncurkan minimal 1 produk pertama dulu.");
        }
        if status.level < 2 {
            return Err("Capai Level Founder 2 terlebih dahulu.");
        }
        if market.active_users < 20 {
            return Err("Dapatkan minimal 20 pelanggan aktif terlebih dahulu.");
        }
        if finance.cash_balance < STAGE_2_UPGRADE_COST {
            return Err("Kas butuh minimal $3,500 untuk pindah ke Small Startup.");
        }
        Ok(())
    }

    pub fn can_upgrade_to_stage_3(
        &self,
        status: &PlayerStatus,
        employees: &EmployeeState,
        market: &MarketState,
        finance: &FinanceState,
    ) -> Result<(), &'static str> {
        if self.current != CompanyStage::SmallStartup {
            return Err("Harus berada di Stage 2 terlebih dahulu.");
        }
        if status.level < 3 {
            return Err("Capai Level Founder 3 terlebih dahulu.");
        }
        if employees.roster.len() < 2 {
            return Err("Rekrut minimal 2 anggota tim terlebih dahulu.");
        }
        if market.active_users < 60 {
            return Err("Capai minimal 60 pelanggan aktif terlebih dahulu.");
        }
        if finance.cash_balance < STAGE_3_UPGRADE_COST {
            return Err("Kas butuh minimal $8,000 untuk membuka Kantor & Lobby Stage 3.");
        }
        Ok(())
    }

    pub fn can_upgrade_to_stage_4(
        &self,
        status: &PlayerStatus,
        employees: &EmployeeState,
        product: &ProductState,
        market: &MarketState,
        finance: &FinanceState,
    ) -> Result<(), &'static str> {
        if self.current != CompanyStage::GrowingCompany {
            return Err("Harus berada di Stage 3 terlebih dahulu.");
        }
        if status.level < 4 {
            return Err("Capai Level Founder 4 terlebih dahulu.");
        }
        if !employees.has_manager() {
            return Err("Rekrut 1 Manajer Kantor di tab Tim terlebih dahulu.");
        }
        if product.launched_count() < 2 {
            return Err("Miliki minimal 2 produk yang sudah dipasarkan.");
        }
        if market.active_users < 140 {
            return Err("Capai minimal 140 pelanggan aktif terlebih dahulu.");
        }
        if finance.cash_balance < STAGE_4_UPGRADE_COST {
            return Err("Kas butuh minimal $18,000 untuk ekspansi ke Stage 4: Corporation.");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodItemKind {
    Coffee,
    SnackBox,
    LunchBento,
    TeamSnack,
}

impl FoodItemKind {
    pub const ALL: [FoodItemKind; 4] = [
        FoodItemKind::Coffee,
        FoodItemKind::SnackBox,
        FoodItemKind::LunchBento,
        FoodItemKind::TeamSnack,
    ];

    pub fn label(self) -> &'static str {
        match self {
            FoodItemKind::Coffee => "Kopi Susu Hangat",
            FoodItemKind::SnackBox => "Roti & Camilan",
            FoodItemKind::LunchBento => "Paket Makan Siang",
            FoodItemKind::TeamSnack => "Paket Kopi & Snack Tim",
        }
    }

    pub fn cost(self) -> i64 {
        match self {
            FoodItemKind::Coffee => 12,
            FoodItemKind::SnackBox => 18,
            FoodItemKind::LunchBento => 30,
            FoodItemKind::TeamSnack => 65,
        }
    }

    pub fn effect_summary(self) -> &'static str {
        match self {
            FoodItemKind::Coffee => "+28 Energi, +8 Kenyang, +10 Semangat",
            FoodItemKind::SnackBox => "+10 Energi, +32 Kenyang, +6 Semangat",
            FoodItemKind::LunchBento => "+22 Energi, +58 Kenyang, +14 Semangat",
            FoodItemKind::TeamSnack => "+20 Energi & +30 Kenyang + Segarkan Tim!",
        }
    }

    pub fn energy_restore(self) -> f32 {
        match self {
            FoodItemKind::Coffee => 28.0,
            FoodItemKind::SnackBox => 10.0,
            FoodItemKind::LunchBento => 22.0,
            FoodItemKind::TeamSnack => 20.0,
        }
    }

    pub fn hunger_restore(self) -> f32 {
        match self {
            FoodItemKind::Coffee => 8.0,
            FoodItemKind::SnackBox => 32.0,
            FoodItemKind::LunchBento => 58.0,
            FoodItemKind::TeamSnack => 30.0,
        }
    }

    pub fn mood_restore(self) -> f32 {
        match self {
            FoodItemKind::Coffee => 10.0,
            FoodItemKind::SnackBox => 6.0,
            FoodItemKind::LunchBento => 14.0,
            FoodItemKind::TeamSnack => 18.0,
        }
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct PlayerStatus {
    pub energy: f32,
    pub hunger: f32,
    pub mood: f32,
    pub level: u32,
    pub xp: u32,
    pub snack_bag: Vec<FoodItemKind>,
    pub deliveries_received: u32,
    pub products_launched: u32,
    pub npc_chats_count: u32,
    #[serde(default)]
    pub low_energy_warned: bool,
    #[serde(default)]
    pub low_hunger_warned: bool,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            energy: 92.0,
            hunger: 85.0,
            mood: 82.0,
            level: 1,
            xp: 0,
            snack_bag: vec![FoodItemKind::Coffee],
            deliveries_received: 0,
            products_launched: 0,
            npc_chats_count: 0,
            low_energy_warned: false,
            low_hunger_warned: false,
        }
    }
}

impl PlayerStatus {
    pub fn xp_to_next_level(&self) -> u32 {
        100 * self.level.max(1)
    }

    /// Adds XP and returns true if the founder gained at least one level.
    pub fn gain_xp(&mut self, amount: u32) -> bool {
        self.xp = self.xp.saturating_add(amount);
        let mut leveled = false;
        while self.xp >= self.xp_to_next_level() {
            self.xp -= self.xp_to_next_level();
            self.level += 1;
            leveled = true;
        }
        leveled
    }

    pub fn work_efficiency(&self) -> f32 {
        let level_bonus = 1.0 + (self.level.saturating_sub(1) as f32) * 0.08;
        if self.energy < 25.0 || self.hunger < 20.0 {
            level_bonus * 0.55
        } else if self.mood >= 75.0 {
            level_bonus * 1.15
        } else {
            level_bonus
        }
    }

    pub fn mood_label(&self) -> &'static str {
        if self.mood >= 75.0 {
            "Bersemangat"
        } else if self.mood >= 45.0 {
            "Normal"
        } else {
            "Stres / Lelah"
        }
    }

    pub fn apply_food(&mut self, item: FoodItemKind) {
        self.energy = (self.energy + item.energy_restore()).clamp(0.0, 100.0);
        self.hunger = (self.hunger + item.hunger_restore()).clamp(0.0, 100.0);
        self.mood = (self.mood + item.mood_restore()).clamp(0.0, 100.0);
        if self.energy >= 30.0 {
            self.low_energy_warned = false;
        }
        if self.hunger >= 30.0 {
            self.low_hunger_warned = false;
        }
    }

    pub fn rest(&mut self, energy_gain: f32, mood_gain: f32) {
        self.energy = (self.energy + energy_gain).clamp(0.0, 100.0);
        self.mood = (self.mood + mood_gain).clamp(0.0, 100.0);
        if self.energy >= 30.0 {
            self.low_energy_warned = false;
        }
    }

    /// Simulates one in-game hour of hunger and energy drain.
    /// Returns optional notification `(text, is_warning)`.
    pub fn tick_hour(&mut self, working_hard: bool) -> Option<(String, bool)> {
        let hunger_drain = if working_hard { 3.4 } else { 2.3 };
        self.hunger = (self.hunger - hunger_drain).max(0.0);

        let mut energy_drain = if working_hard { 4.1 } else { 2.0 };
        if self.hunger < 25.0 {
            energy_drain += 2.0;
            self.mood = (self.mood - 1.5).max(10.0);
        } else {
            self.mood = (self.mood - 0.3).max(15.0);
        }

        self.energy = (self.energy - energy_drain).max(0.0);

        if self.energy <= 0.0 {
            self.energy = 58.0;
            self.hunger = (self.hunger - 10.0).max(5.0);
            self.mood = (self.mood - 10.0).max(10.0);
            self.low_energy_warned = false;
            return Some((
                "Kamu kelelahan dan ketiduran sejenak! Pesan makan atau rekrut tim agar tidak kewalahan."
                    .to_string(),
                true,
            ));
        }

        if self.hunger < 25.0 && !self.low_hunger_warned {
            self.low_hunger_warned = true;
            return Some((
                "Perut lapar! Pesan makanan lewat tombol Order / Kurir atau makan dari tas."
                    .to_string(),
                true,
            ));
        }

        if self.energy < 25.0 && !self.low_energy_warned {
            self.low_energy_warned = true;
            return Some((
                "Energi mulai menipis! Istirahat sejenak atau minum kopi agar kerja tetap cepat."
                    .to_string(),
                true,
            ));
        }

        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryPayload {
    Food(FoodItemKind),
    Equipment(OfficeUpgradeKind),
}

impl DeliveryPayload {
    pub fn label(self) -> &'static str {
        match self {
            DeliveryPayload::Food(f) => f.label(),
            DeliveryPayload::Equipment(e) => e.label(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeliveryOrder {
    pub id: u32,
    pub payload: DeliveryPayload,
    pub eta_secs: f32,
    pub arrived: bool,
}

#[derive(Resource, Clone, Default, Debug, Serialize, Deserialize)]
pub struct DeliveryState {
    pub next_id: u32,
    pub orders: Vec<DeliveryOrder>,
}

impl DeliveryState {
    pub fn has_pending_equipment(&self, kind: OfficeUpgradeKind) -> bool {
        self.orders
            .iter()
            .any(|o| matches!(o.payload, DeliveryPayload::Equipment(k) if k == kind))
    }

    pub fn place_food_order(
        &mut self,
        item: FoodItemKind,
        day: u32,
        finance: &mut FinanceState,
    ) -> Result<u32, &'static str> {
        let cost = item.cost();
        if finance.cash_balance < cost {
            return Err("Uang kas tidak cukup untuk memesan makanan ini.");
        }
        finance.record_transaction(
            day,
            format!("Order: {}", item.label()),
            -cost,
            TransactionKind::OfficeUpgrade,
        );
        self.next_id += 1;
        let id = self.next_id;
        self.orders.push(DeliveryOrder {
            id,
            payload: DeliveryPayload::Food(item),
            eta_secs: 2.5,
            arrived: false,
        });
        Ok(id)
    }

    pub fn place_equipment_order(
        &mut self,
        kind: OfficeUpgradeKind,
        day: u32,
        finance: &mut FinanceState,
        expansion: &OfficeExpansionState,
    ) -> Result<u32, &'static str> {
        if expansion.is_purchased(kind) {
            return Err("Barang kantor ini sudah dimiliki.");
        }
        if self.has_pending_equipment(kind) {
            return Err("Barang ini sedang dalam perjalanan diantar kurir.");
        }
        let cost = kind.cost();
        if finance.cash_balance < cost {
            return Err("Kas tidak cukup untuk membeli perlengkapan kantor ini.");
        }
        finance.record_transaction(
            day,
            format!("Beli: {}", kind.label()),
            -cost,
            TransactionKind::OfficeUpgrade,
        );
        self.next_id += 1;
        let id = self.next_id;
        self.orders.push(DeliveryOrder {
            id,
            payload: DeliveryPayload::Equipment(kind),
            eta_secs: 3.0,
            arrived: false,
        });
        Ok(id)
    }
}

#[derive(Component)]
pub struct CourierNpc {
    pub arriving: bool,
    pub leaving: bool,
    pub target_pos: Vec3,
    pub notified_arrival: bool,
    pub active_anim: Option<usize>,
}

#[derive(Component)]
pub struct CourierAnimPlayer;

#[derive(Resource)]
struct CourierAnimations {
    graph: Handle<AnimationGraph>,
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
}

fn setup_courier_anim_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let clips = [
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(COURIER_ASSET)),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(COURIER_ASSET)),
    ];
    let (graph, nodes) = AnimationGraph::from_clips(clips);
    commands.insert_resource(CourierAnimations {
        graph: graphs.add(graph),
        idle: nodes[0],
        walk: nodes[1],
    });
}

#[derive(Message, Clone, Copy, Debug)]
pub enum LifeCommand {
    OrderFood(FoodItemKind),
    OrderEquipment(OfficeUpgradeKind),
    EatFromBag(usize),
    RestFounder,
    UpgradeToStage2,
    UpgradeToStage3,
    UpgradeToStage4,
}

#[derive(Clone, Debug)]
pub struct QuestItemView {
    pub id: &'static str,
    pub title: &'static str,
    pub detail: &'static str,
    pub xp_reward: u32,
    pub done: bool,
}

#[derive(Resource, Clone, Default, Debug, Serialize, Deserialize)]
pub struct QuestState {
    pub rewarded_ids: HashSet<String>,
}

impl QuestState {
    pub fn build_quest_list(
        &self,
        stage: CompanyStage,
        status: &PlayerStatus,
        product: &ProductState,
        employees: &EmployeeState,
        market: &MarketState,
        expansion: &OfficeExpansionState,
    ) -> Vec<QuestItemView> {
        let has_project = product.active.is_some() || status.products_launched >= 1;
        let launched = status.products_launched >= 1
            || product
                .active
                .as_ref()
                .is_some_and(|p| p.phase == ProductPhase::Launched);

        let mut list = vec![
            QuestItemView {
                id: "q1_first_idea",
                title: "Mulai Rencana Usaha",
                detail: "Buka 'Kelola Usaha' & pilih ide produk pertama.",
                xp_reward: 45,
                done: has_project,
            },
            QuestItemView {
                id: "q2_courier_order",
                title: "Pesan & Terima Paket Kurir",
                detail: "Pesan makan/barang di 'Order / Kurir', lalu ambil di pintu [E].",
                xp_reward: 40,
                done: status.deliveries_received >= 1,
            },
            QuestItemView {
                id: "q3_launch_product",
                title: "Luncurkan Produk Pertama",
                detail: "Tunggu pembuatan & cek kualitas 100%, lalu tekan Rilis.",
                xp_reward: 80,
                done: launched,
            },
            QuestItemView {
                id: "q4_hire_first",
                title: "Rekrut Karyawan Pertama",
                detail: "Ajak minimal 1 rekan bergabung di tab Tim.",
                xp_reward: 65,
                done: !employees.roster.is_empty(),
            },
            QuestItemView {
                id: "q5_stage_2",
                title: "Naik ke Stage 2: Small Startup",
                detail: "Capai Level 2, 20+ pelanggan & kas $3,500 lalu upgrade Stage.",
                xp_reward: 120,
                done: stage != CompanyStage::Founder,
            },
        ];

        if stage == CompanyStage::SmallStartup {
            list.push(QuestItemView {
                id: "q6_office_props",
                title: "Lengkapi Studio Kantor",
                detail: "Pesan minimal 2 perlengkapan kantor lewat kurir.",
                xp_reward: 60,
                done: expansion.purchased.len() >= 2,
            });
            list.push(QuestItemView {
                id: "q7_talk_team",
                title: "Sapa & Cek Tim Langsung",
                detail: "Hampiri karyawan & tekan [E] untuk cek kondisi.",
                xp_reward: 45,
                done: status.npc_chats_count >= 1,
            });
            list.push(QuestItemView {
                id: "q8_stage_3",
                title: "Naik ke Stage 3: Growing Company",
                detail: "Capai Level 3, 2 tim, 60 pelanggan & $8,000 lalu upgrade.",
                xp_reward: 160,
                done: false,
            });
        }

        if matches!(
            stage,
            CompanyStage::GrowingCompany | CompanyStage::Corporation
        ) {
            list.push(QuestItemView {
                id: "q9_second_product",
                title: "Kembangkan Produk Ke-2",
                detail: "Simpan produk lama ke portofolio & luncurkan produk ke-2.",
                xp_reward: 140,
                done: product.launched_count() >= 2,
            });
            list.push(QuestItemView {
                id: "q10_hire_manager",
                title: "Rekrut Manajer Kantor",
                detail: "Pekerjakan Manajer di tab Tim agar stamina tim terjaga.",
                xp_reward: 110,
                done: employees.has_manager(),
            });
            list.push(QuestItemView {
                id: "q11_stage_4",
                title: "Capai Stage 4: Corporation",
                detail: "Capai Level 4, 2 produk rilis, Manajer, 140 pelanggan & $18,000.",
                xp_reward: 250,
                done: stage == CompanyStage::Corporation,
            });
            list.push(QuestItemView {
                id: "q12_market_leader",
                title: "Kuasai Pasar (250+ Pelanggan)",
                detail: "Kalahkan pesaing dengan promosi, kualitas & tim lengkap!",
                xp_reward: 300,
                done: market.active_users >= 250,
            });
        }

        list
    }
}

fn spawn_courier_entity(
    commands: &mut Commands,
    asset_server: &AssetServer,
    stage: CompanyStage,
) {
    let uses_lobby = matches!(
        stage,
        CompanyStage::GrowingCompany | CompanyStage::Corporation
    );
    let (spawn_pos, target_pos) = if uses_lobby {
        (Vec3::new(8.65, 0.0, 4.35), Vec3::new(8.65, 0.0, 1.85))
    } else {
        (Vec3::new(0.88, 0.0, 4.35), Vec3::new(0.88, 0.0, 2.75))
    };

    commands.spawn((
        CourierNpc {
            arriving: true,
            leaving: false,
            target_pos,
            notified_arrival: false,
            active_anim: None,
        },
        Transform::from_translation(spawn_pos),
        Visibility::default(),
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(COURIER_ASSET))),
    ));
}

fn has_courier_ancestor(
    mut current: Entity,
    parents: &Query<&ChildOf>,
    couriers: &Query<(), With<CourierNpc>>,
) -> Option<Entity> {
    loop {
        if couriers.contains(current) {
            return Some(current);
        }
        match parents.get(current) {
            Ok(child_of) => current = child_of.parent(),
            Err(_) => return None,
        }
    }
}

fn setup_courier_animations(
    mut commands: Commands,
    anims: Res<CourierAnimations>,
    parents: Query<&ChildOf>,
    couriers: Query<(), With<CourierNpc>>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        if has_courier_ancestor(entity, &parents, &couriers).is_some() {
            let mut transitions = AnimationTransitions::new();
            transitions
                .play(&mut player, anims.walk, Duration::ZERO)
                .repeat();
            commands.entity(entity).insert((
                CourierAnimPlayer,
                AnimationGraphHandle(anims.graph.clone()),
                transitions,
            ));
        }
    }
}

fn sync_courier_animations(
    anims: Res<CourierAnimations>,
    parents: Query<&ChildOf>,
    mut courier_q: Query<&mut CourierNpc>,
    mut anim_q: Query<
        (Entity, &mut AnimationPlayer, &mut AnimationTransitions),
        With<CourierAnimPlayer>,
    >,
) {
    for (anim_entity, mut anim_player, mut transitions) in &mut anim_q {
        let mut cur = anim_entity;
        let mut owner = None;
        while let Ok(child_of) = parents.get(cur) {
            cur = child_of.parent();
            if courier_q.contains(cur) {
                owner = Some(cur);
                break;
            }
        }
        let Some(c_ent) = owner else {
            continue;
        };
        let Ok(mut npc) = courier_q.get_mut(c_ent) else {
            continue;
        };
        let target_idx = if npc.arriving || npc.leaving { 1 } else { 0 };
        if npc.active_anim != Some(target_idx) {
            let node = if target_idx == 1 { anims.walk } else { anims.idle };
            transitions
                .play(&mut anim_player, node, Duration::from_millis(160))
                .repeat();
            npc.active_anim = Some(target_idx);
        }
    }
}

fn boost_team_with_snack(employees: &mut EmployeeState) {
    for emp in &mut employees.roster {
        emp.energy = (emp.energy + 25.0).clamp(0.0, 100.0);
        emp.mood = (emp.mood + 22.0).clamp(0.0, 100.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_life_commands(
    mut reader: MessageReader<LifeCommand>,
    clock: Res<GameClock>,
    mut stage: ResMut<CompanyStageState>,
    mut status: ResMut<PlayerStatus>,
    mut delivery: ResMut<DeliveryState>,
    mut finance: ResMut<FinanceState>,
    mut company: ResMut<CompanyState>,
    mut employees: ResMut<EmployeeState>,
    product: Res<ProductState>,
    market: Res<MarketState>,
    expansion: Res<OfficeExpansionState>,
    mut notify: MessageWriter<NotifyMessage>,
    mut pickup_sfx: MessageWriter<PickupSoundEvent>,
) {
    for cmd in reader.read() {
        match cmd {
            LifeCommand::OrderFood(item) => {
                match delivery.place_food_order(*item, clock.day, &mut finance) {
                    Ok(_) => {
                        notify.write(NotifyMessage::info(format!(
                            "Pesanan '{}' dikirim kurir! Tunggu di depan pintu.",
                            item.label()
                        )));
                    }
                    Err(err) => {
                        notify.write(NotifyMessage::warn(err));
                    }
                }
            }
            LifeCommand::OrderEquipment(kind) => {
                match delivery.place_equipment_order(*kind, clock.day, &mut finance, &expansion) {
                    Ok(_) => {
                        notify.write(NotifyMessage::info(format!(
                            "Kurir sedang mengantar '{}' ke depan pintu!",
                            kind.label()
                        )));
                    }
                    Err(err) => {
                        notify.write(NotifyMessage::warn(err));
                    }
                }
            }
            LifeCommand::EatFromBag(idx) => {
                if *idx < status.snack_bag.len() {
                    let item = status.snack_bag.remove(*idx);
                    status.apply_food(item);
                    if item == FoodItemKind::TeamSnack {
                        boost_team_with_snack(&mut employees);
                    }
                    pickup_sfx.write(PickupSoundEvent);
                    notify.write(NotifyMessage::info(format!(
                        "Menikmati {}! Energi & kondisi tubuh pulih.",
                        item.label()
                    )));
                }
            }
            LifeCommand::RestFounder => {
                status.rest(35.0, 10.0);
                notify.write(NotifyMessage::info(
                    "Istirahat sebentar selesai. Energi +35 & Semangat +10!",
                ));
            }
            LifeCommand::UpgradeToStage2 => {
                match stage.can_upgrade_to_stage_2(&status, &product, &market, &finance) {
                    Ok(()) => {
                        finance.record_transaction(
                            clock.day,
                            "Pindah ke Studio Stage 2 (Small Startup)",
                            -STAGE_2_UPGRADE_COST,
                            TransactionKind::OfficeUpgrade,
                        );
                        stage.current = CompanyStage::SmallStartup;
                        company.office_tier = CompanyStage::SmallStartup.place_label().to_string();
                        company.reputation = (company.reputation + 10.0).min(100.0);
                        employees.max_employees = employees.max_employees.max(2);
                        let leveled = status.gain_xp(120);
                        pickup_sfx.write(PickupSoundEvent);
                        notify.write(NotifyMessage::info(
                            "Selamat! Perusahaan resmi naik ke Stage 2: Small Startup!",
                        ));
                        if leveled {
                            notify.write(NotifyMessage::info(format!(
                                "Level Founder naik ke Level {}!",
                                status.level
                            )));
                        }
                    }
                    Err(err) => {
                        notify.write(NotifyMessage::warn(err));
                    }
                }
            }
            LifeCommand::UpgradeToStage3 => {
                match stage.can_upgrade_to_stage_3(&status, &employees, &market, &finance) {
                    Ok(()) => {
                        finance.record_transaction(
                            clock.day,
                            "Ekspansi ke Stage 3 (Growing Company + Lobby)",
                            -STAGE_3_UPGRADE_COST,
                            TransactionKind::OfficeUpgrade,
                        );
                        stage.current = CompanyStage::GrowingCompany;
                        company.office_tier =
                            CompanyStage::GrowingCompany.place_label().to_string();
                        company.reputation = (company.reputation + 14.0).min(100.0);
                        employees.max_employees = employees.max_employees.max(4);
                        let leveled = status.gain_xp(180);
                        pickup_sfx.write(PickupSoundEvent);
                        notify.write(NotifyMessage::info(
                            "Stage 3 Terbuka: Growing Company! Lobby Resepsionis & 4 slot tim kini aktif!",
                        ));
                        if leveled {
                            notify.write(NotifyMessage::info(format!(
                                "Level Founder naik ke Level {}!",
                                status.level
                            )));
                        }
                    }
                    Err(err) => {
                        notify.write(NotifyMessage::warn(err));
                    }
                }
            }
            LifeCommand::UpgradeToStage4 => {
                match stage.can_upgrade_to_stage_4(
                    &status,
                    &employees,
                    &product,
                    &market,
                    &finance,
                ) {
                    Ok(()) => {
                        finance.record_transaction(
                            clock.day,
                            "Ekspansi Korporasi Stage 4",
                            -STAGE_4_UPGRADE_COST,
                            TransactionKind::OfficeUpgrade,
                        );
                        stage.current = CompanyStage::Corporation;
                        company.office_tier = CompanyStage::Corporation.place_label().to_string();
                        company.reputation = (company.reputation + 20.0).min(100.0);
                        employees.max_employees = employees.max_employees.max(5);
                        let leveled = status.gain_xp(260);
                        pickup_sfx.write(PickupSoundEvent);
                        notify.write(NotifyMessage::info(
                            "Pencapaian Tertinggi! Usahamu resmi menjadi Stage 4: Corporation!",
                        ));
                        if leveled {
                            notify.write(NotifyMessage::info(format!(
                                "Level Founder naik ke Level {}!",
                                status.level
                            )));
                        }
                    }
                    Err(err) => {
                        notify.write(NotifyMessage::warn(err));
                    }
                }
            }
        }
    }
}

fn tick_player_needs_hourly(
    mut hour_reader: MessageReader<HourPassedMessage>,
    product: Res<ProductState>,
    mut status: ResMut<PlayerStatus>,
    mut notify: MessageWriter<NotifyMessage>,
) {
    let working_hard = product.active.as_ref().is_some_and(|p| {
        matches!(p.phase, ProductPhase::Development | ProductPhase::Testing)
    });

    for _ in hour_reader.read() {
        if let Some((text, warn)) = status.tick_hour(working_hard) {
            if warn {
                notify.write(NotifyMessage::warn(text));
            } else {
                notify.write(NotifyMessage::info(text));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn advance_deliveries_and_couriers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    clock: Res<GameClock>,
    stage: Res<CompanyStageState>,
    mut delivery: ResMut<DeliveryState>,
    mut courier_q: Query<(Entity, &mut Transform, &mut CourierNpc)>,
    mut notify: MessageWriter<NotifyMessage>,
) {
    let dt = time.delta_secs();
    let sim_dt = dt * clock.speed.multiplier().max(0.5);
    let mut any_arrived_order = false;

    for order in &mut delivery.orders {
        if !order.arrived {
            order.eta_secs -= sim_dt;
            if order.eta_secs <= 0.0 {
                order.eta_secs = 0.0;
                order.arrived = true;
                any_arrived_order = true;
            }
        } else {
            any_arrived_order = true;
        }
    }

    if any_arrived_order && courier_q.is_empty() {
        spawn_courier_entity(&mut commands, &asset_server, stage.current);
    }

    let uses_lobby = matches!(
        stage.current,
        CompanyStage::GrowingCompany | CompanyStage::Corporation
    );

    for (entity, mut tf, mut npc) in &mut courier_q {
        if npc.leaving {
            let exit_target = Vec3::new(tf.translation.x, 0.0, 4.65);
            let diff = exit_target - tf.translation;
            if diff.length() < 0.08 {
                commands.entity(entity).despawn();
            } else {
                tf.translation += diff.normalize() * 2.1 * dt;
                tf.rotation = Quat::from_rotation_y(PI);
            }
            continue;
        }

        if npc.arriving {
            let diff = npc.target_pos - tf.translation;
            if diff.length() < 0.06 {
                tf.translation = npc.target_pos;
                npc.arriving = false;
                if !npc.notified_arrival {
                    npc.notified_arrival = true;
                    commands.entity(entity).insert(Interactable {
                        kind: InteractableKind::Courier,
                        prompt: if uses_lobby {
                            "Ambil Pesanan dari Kurir di Lobby".into()
                        } else {
                            "Ambil Pesanan dari Kurir".into()
                        },
                        range: 2.9,
                        offset: Vec3::new(0.0, 1.0, 0.0),
                    });
                    notify.write(NotifyMessage::info(if uses_lobby {
                        "Kurir tiba di Lobby Resepsionis! Hampiri & tekan [E] untuk ambil paket."
                    } else {
                        "Ting-tong! Kurir tiba di depan pintu. Dekati & tekan [E] untuk ambil pesanan."
                    }));
                }
            } else {
                tf.translation += diff.normalize() * 1.9 * dt;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_courier_and_sofa_interactions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut reader: MessageReader<InteractTriggered>,
    clock: Res<GameClock>,
    mut delivery: ResMut<DeliveryState>,
    mut status: ResMut<PlayerStatus>,
    mut expansion: ResMut<OfficeExpansionState>,
    mut colliders: ResMut<OfficeColliders>,
    mut company: ResMut<CompanyState>,
    mut employees: ResMut<EmployeeState>,
    mut courier_q: Query<(Entity, &mut CourierNpc)>,
    mut notify: MessageWriter<NotifyMessage>,
    mut pickup_sfx: MessageWriter<PickupSoundEvent>,
) {
    for ev in reader.read() {
        match ev.kind {
            InteractableKind::Courier => {
                let mut collected_labels = Vec::new();
                let mut remaining = Vec::new();
                let orders = std::mem::take(&mut delivery.orders);
                for order in orders {
                    if order.arrived {
                        collected_labels.push(order.payload.label().to_string());
                        match order.payload {
                            DeliveryPayload::Food(food) => {
                                if status.snack_bag.len() < 12 {
                                    status.snack_bag.push(food);
                                } else {
                                    status.apply_food(food);
                                    if food == FoodItemKind::TeamSnack {
                                        boost_team_with_snack(&mut employees);
                                    }
                                }
                            }
                            DeliveryPayload::Equipment(kind) => {
                                if !expansion.is_purchased(kind) {
                                    expansion.purchased.push(kind);
                                    company.reputation =
                                        (company.reputation + kind.rep_bonus()).min(100.0);
                                    if kind == OfficeUpgradeKind::ExtraWorkstation {
                                        employees.max_employees = employees.max_employees.max(2);
                                    }
                                    spawn_upgrade_visual(&mut commands, &asset_server, kind);
                                    rebuild_colliders(&mut colliders, &expansion);
                                }
                            }
                        }
                    } else {
                        remaining.push(order);
                    }
                }
                delivery.orders = remaining;

                if !collected_labels.is_empty() {
                    status.deliveries_received += 1;
                    let leveled = status.gain_xp(15);
                    pickup_sfx.write(PickupSoundEvent);
                    notify.write(NotifyMessage::info(format!(
                        "Hari {}: Paket diterima ({})!",
                        clock.day,
                        collected_labels.join(", ")
                    )));
                    if leveled {
                        notify.write(NotifyMessage::info(format!(
                            "Level Founder naik ke Level {}!",
                            status.level
                        )));
                    }
                }

                for (c_ent, mut npc) in &mut courier_q {
                    npc.leaving = true;
                    commands.entity(c_ent).remove::<Interactable>();
                }
            }
            InteractableKind::Sofa => {
                status.rest(42.0, 14.0);
                pickup_sfx.write(PickupSoundEvent);
                notify.write(NotifyMessage::info(
                    "Istirahat di sofa sangat nyaman! Energi +42 & Semangat +14.",
                ));
            }
            _ => {}
        }
    }
}

fn evaluate_quests_and_milestones(
    stage: Res<CompanyStageState>,
    mut status: ResMut<PlayerStatus>,
    product: Res<ProductState>,
    employees: Res<EmployeeState>,
    market: Res<MarketState>,
    expansion: Res<OfficeExpansionState>,
    mut quests: ResMut<QuestState>,
    mut notify: MessageWriter<NotifyMessage>,
) {
    if product
        .active
        .as_ref()
        .is_some_and(|p| p.phase == ProductPhase::Launched)
        && status.products_launched == 0
    {
        status.products_launched = 1;
    }

    let items = quests.build_quest_list(
        stage.current,
        &status,
        &product,
        &employees,
        &market,
        &expansion,
    );

    let mut newly_completed = Vec::new();
    for item in items {
        if item.done && !quests.rewarded_ids.contains(item.id) {
            newly_completed.push((item.id.to_string(), item.title, item.xp_reward));
        }
    }

    for (id, title, xp_reward) in newly_completed {
        quests.rewarded_ids.insert(id);
        let leveled = status.gain_xp(xp_reward);
        notify.write(NotifyMessage::info(format!(
            "Misi Selesai: {title} (+{xp_reward} XP)!"
        )));
        if leveled {
            notify.write(NotifyMessage::info(format!(
                "Selamat! Kamu naik ke Level Founder {}!",
                status.level
            )));
        }
    }
}

/// Shared helper to ensure EmployeeProfile default stats exist on loaded saves.
pub fn ensure_employee_stats_sane(emp: &mut EmployeeProfile) {
    if emp.energy <= 0.0 && emp.mood <= 0.0 {
        emp.energy = 90.0;
        emp.mood = 85.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_needs_drain_and_food_restore() {
        let mut status = PlayerStatus::default();
        assert_eq!(status.level, 1);
        for _ in 0..8 {
            status.tick_hour(true);
        }
        assert!(status.energy < 92.0);
        assert!(status.hunger < 85.0);

        let e_before = status.energy;
        let h_before = status.hunger;
        status.apply_food(FoodItemKind::LunchBento);
        assert!(status.energy > e_before);
        assert!(status.hunger > h_before);
    }

    #[test]
    fn food_and_equipment_delivery_deducts_cash_and_queues() {
        let mut finance = FinanceState::default();
        let mut delivery = DeliveryState::default();
        let expansion = OfficeExpansionState::default();

        delivery
            .place_food_order(FoodItemKind::Coffee, 1, &mut finance)
            .unwrap();
        assert_eq!(finance.cash_balance, 25_000 - 12);
        assert_eq!(delivery.orders.len(), 1);

        delivery
            .place_equipment_order(OfficeUpgradeKind::OfficePlant, 1, &mut finance, &expansion)
            .unwrap();
        assert_eq!(finance.cash_balance, 25_000 - 12 - 150);
        assert_eq!(delivery.orders.len(), 2);
    }

    #[test]
    fn xp_gain_levels_up_founder() {
        let mut status = PlayerStatus::default();
        assert!(!status.gain_xp(50));
        assert_eq!(status.level, 1);
        assert!(status.gain_xp(70));
        assert_eq!(status.level, 2);
        assert_eq!(status.xp, 20);
    }
}
