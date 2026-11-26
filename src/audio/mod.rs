use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

use crate::{
    core::NotifyMessage,
    employee::{EmployeeCommand, EmployeeState, EmployeeWorkState},
    interaction::{InteractTriggered, InteractableKind},
    market::MarketCommand,
    office::OfficeUpgradeCommand,
    player::PlayerController,
    product::ProductCommand,
    save_system::SaveGameCommand,
};

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioSettings>()
            .add_systems(Startup, setup_audio_assets)
            .add_systems(
                Update,
                (
                    toggle_mute_hotkey,
                    sync_ambient_mute,
                    play_interaction_sfx,
                    play_command_sfx,
                    play_notify_sfx,
                    play_footstep_sfx,
                    play_typing_sfx,
                ),
            );
    }
}

#[derive(Resource)]
pub struct AudioSettings {
    pub muted: bool,
    step_timer: f32,
    typing_timer: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            muted: false,
            step_timer: 0.0,
            typing_timer: 0.0,
        }
    }
}

#[derive(Resource)]
struct SoundBank {
    ui_click: Handle<AudioSource>,
    door: Handle<AudioSource>,
    notify: Handle<AudioSource>,
    cash: Handle<AudioSource>,
    step: Handle<AudioSource>,
    typing: Handle<AudioSource>,
}

#[derive(Component)]
struct AmbientOfficeAudio;

fn setup_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bank = SoundBank {
        ui_click: asset_server.load("audio/sfx/sfx_ui_click.ogg"),
        door: asset_server.load("audio/sfx/sfx_door.ogg"),
        notify: asset_server.load("audio/sfx/sfx_notify.ogg"),
        cash: asset_server.load("audio/sfx/sfx_cash.ogg"),
        step: asset_server.load("audio/sfx/sfx_step.ogg"),
        typing: asset_server.load("audio/sfx/sfx_typing.ogg"),
    };
    commands.insert_resource(bank);

    commands.spawn((
        AmbientOfficeAudio,
        AudioPlayer::new(asset_server.load("audio/ambient/ambient_office.ogg")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::Linear(0.18),
            ..default()
        },
    ));
}

fn spawn_oneshot(commands: &mut Commands, source: Handle<AudioSource>, vol: f32, muted: bool) {
    if muted {
        return;
    }
    commands.spawn((
        AudioPlayer::new(source),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(vol),
            ..default()
        },
    ));
}

fn toggle_mute_hotkey(keys: Res<ButtonInput<KeyCode>>, mut settings: ResMut<AudioSettings>) {
    if keys.just_pressed(KeyCode::KeyM) {
        settings.muted = !settings.muted;
    }
}

fn sync_ambient_mute(
    settings: Res<AudioSettings>,
    mut ambient_q: Query<&mut AudioSink, With<AmbientOfficeAudio>>,
) {
    if !settings.is_changed() {
        return;
    }
    for mut sink in &mut ambient_q {
        let vol = if settings.muted { 0.0 } else { 0.18 };
        sink.set_volume(Volume::Linear(vol));
    }
}

fn play_interaction_sfx(
    mut commands: Commands,
    mut reader: MessageReader<InteractTriggered>,
    bank: Res<SoundBank>,
    settings: Res<AudioSettings>,
) {
    for ev in reader.read() {
        match ev.kind {
            InteractableKind::Door => {
                spawn_oneshot(&mut commands, bank.door.clone(), 0.45, settings.muted);
            }
            InteractableKind::Workstation => {
                spawn_oneshot(&mut commands, bank.ui_click.clone(), 0.40, settings.muted);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn play_command_sfx(
    mut commands: Commands,
    mut prod_r: MessageReader<ProductCommand>,
    mut emp_r: MessageReader<EmployeeCommand>,
    mut off_r: MessageReader<OfficeUpgradeCommand>,
    mut mkt_r: MessageReader<MarketCommand>,
    mut save_r: MessageReader<SaveGameCommand>,
    bank: Res<SoundBank>,
    settings: Res<AudioSettings>,
) {
    let mut play_cash = false;
    let mut play_click = false;

    for _ in prod_r.read() {
        play_cash = true;
    }
    for _ in emp_r.read() {
        play_cash = true;
    }
    for _ in off_r.read() {
        play_cash = true;
    }
    for _ in mkt_r.read() {
        play_click = true;
    }
    for _ in save_r.read() {
        play_click = true;
    }

    if play_cash {
        spawn_oneshot(&mut commands, bank.cash.clone(), 0.40, settings.muted);
    } else if play_click {
        spawn_oneshot(&mut commands, bank.ui_click.clone(), 0.35, settings.muted);
    }
}

fn play_notify_sfx(
    mut commands: Commands,
    mut reader: MessageReader<NotifyMessage>,
    bank: Res<SoundBank>,
    settings: Res<AudioSettings>,
) {
    if reader.read().next().is_some() {
        spawn_oneshot(&mut commands, bank.notify.clone(), 0.25, settings.muted);
    }
}

fn play_footstep_sfx(
    mut commands: Commands,
    time: Res<Time>,
    player_q: Query<&PlayerController>,
    bank: Res<SoundBank>,
    mut settings: ResMut<AudioSettings>,
) {
    let Ok(ctrl) = player_q.single() else {
        return;
    };
    if !ctrl.is_moving {
        settings.step_timer = 0.08;
        return;
    }
    settings.step_timer -= time.delta_secs();
    if settings.step_timer <= 0.0 {
        settings.step_timer = if ctrl.is_sprinting { 0.28 } else { 0.42 };
        spawn_oneshot(&mut commands, bank.step.clone(), 0.22, settings.muted);
    }
}

fn play_typing_sfx(
    mut commands: Commands,
    time: Res<Time>,
    employees: Res<EmployeeState>,
    bank: Res<SoundBank>,
    mut settings: ResMut<AudioSettings>,
) {
    let any_working = employees
        .roster
        .iter()
        .any(|e| e.state == EmployeeWorkState::Working);
    if !any_working {
        return;
    }
    settings.typing_timer -= time.delta_secs();
    if settings.typing_timer <= 0.0 {
        settings.typing_timer = 1.9;
        spawn_oneshot(&mut commands, bank.typing.clone(), 0.14, settings.muted);
    }
}
