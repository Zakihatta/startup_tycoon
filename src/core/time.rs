use bevy::prelude::*;

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameClock>()
            .add_message::<HourPassedMessage>()
            .add_message::<DayPassedMessage>()
            .add_systems(Update, (handle_speed_hotkeys, advance_game_clock));
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SimSpeed {
    Paused,
    #[default]
    Normal,
    Fast,
}

impl SimSpeed {
    pub fn multiplier(self) -> f32 {
        match self {
            SimSpeed::Paused => 0.0,
            SimSpeed::Normal => 1.0,
            SimSpeed::Fast => 2.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SimSpeed::Paused => "Pause",
            SimSpeed::Normal => "1x",
            SimSpeed::Fast => "2x",
        }
    }
}

/// 1x speed advances 15 in-game minutes every 0.5 real seconds (24h = 48s at 1x).
const REAL_SECONDS_PER_STEP_1X: f32 = 0.5;
const IN_GAME_MINUTES_PER_STEP: u16 = 15;

#[derive(Resource, Debug)]
pub struct GameClock {
    pub day: u32,
    pub hour: u8,
    pub minute: u8,
    pub speed: SimSpeed,
    elapsed_accum: f32,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            day: 1,
            hour: 9,
            minute: 0,
            speed: SimSpeed::Normal,
            elapsed_accum: 0.0,
        }
    }
}

impl GameClock {
    pub fn formatted_time(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    pub fn restore(&mut self, day: u32, hour: u8, minute: u8) {
        self.day = day;
        self.hour = hour;
        self.minute = minute;
        self.elapsed_accum = 0.0;
    }

    /// Advances the clock by `delta_real_secs` * speed multiplier.
    /// Returns `(hours_crossed, days_crossed)`.
    pub fn tick(&mut self, delta_real_secs: f32) -> (Vec<(u32, u8)>, Vec<u32>) {
        let mut hours_crossed = Vec::new();
        let mut days_crossed = Vec::new();

        let mult = self.speed.multiplier();
        if mult <= 0.0 {
            return (hours_crossed, days_crossed);
        }

        self.elapsed_accum += delta_real_secs * mult;
        while self.elapsed_accum >= REAL_SECONDS_PER_STEP_1X {
            self.elapsed_accum -= REAL_SECONDS_PER_STEP_1X;
            let (hour_changed, day_changed) = self.advance_minutes(IN_GAME_MINUTES_PER_STEP);
            if hour_changed {
                hours_crossed.push((self.day, self.hour));
            }
            if day_changed {
                days_crossed.push(self.day);
            }
        }

        (hours_crossed, days_crossed)
    }

    fn advance_minutes(&mut self, minutes: u16) -> (bool, bool) {
        let total_mins = self.minute as u16 + minutes;
        let add_hours = total_mins / 60;
        self.minute = (total_mins % 60) as u8;

        if add_hours == 0 {
            return (false, false);
        }

        let total_hours = self.hour as u16 + add_hours;
        let add_days = (total_hours / 24) as u32;
        self.hour = (total_hours % 24) as u8;

        if add_days > 0 {
            self.day += add_days;
            return (true, true);
        }

        (true, false)
    }
}

#[derive(Message, Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct HourPassedMessage {
    pub day: u32,
    pub hour: u8,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct DayPassedMessage {
    pub new_day: u32,
}

fn handle_speed_hotkeys(keys: Res<ButtonInput<KeyCode>>, mut clock: ResMut<GameClock>) {
    if keys.just_pressed(KeyCode::Digit0) || keys.just_pressed(KeyCode::KeyP) {
        clock.speed = if clock.speed == SimSpeed::Paused {
            SimSpeed::Normal
        } else {
            SimSpeed::Paused
        };
    } else if keys.just_pressed(KeyCode::Digit1) {
        clock.speed = SimSpeed::Normal;
    } else if keys.just_pressed(KeyCode::Digit2) {
        clock.speed = SimSpeed::Fast;
    }
}

fn advance_game_clock(
    time: Res<Time>,
    mut clock: ResMut<GameClock>,
    mut hour_writer: MessageWriter<HourPassedMessage>,
    mut day_writer: MessageWriter<DayPassedMessage>,
) {
    let (hours, days) = clock.tick(time.delta_secs());
    for (day, hour) in hours {
        hour_writer.write(HourPassedMessage { day, hour });
    }
    for new_day in days {
        day_writer.write(DayPassedMessage { new_day });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paused_clock_does_not_advance() {
        let mut clock = GameClock::default();
        clock.speed = SimSpeed::Paused;
        let (hours, days) = clock.tick(10.0);
        assert!(hours.is_empty());
        assert!(days.is_empty());
        assert_eq!(clock.day, 1);
        assert_eq!(clock.hour, 9);
        assert_eq!(clock.minute, 0);
    }

    #[test]
    fn normal_speed_advances_hour_and_rolls_over_day() {
        let mut clock = GameClock::default();
        // 15 hours * 2.0 real secs/hour = 30.0s -> 09:00 + 15h = Day 2 00:00
        let (hours, days) = clock.tick(30.0);
        assert_eq!(hours.len(), 15);
        assert_eq!(days, vec![2]);
        assert_eq!(clock.day, 2);
        assert_eq!(clock.hour, 0);
        assert_eq!(clock.minute, 0);
    }

    #[test]
    fn fast_speed_runs_twice_as_fast() {
        let mut clock = GameClock::default();
        clock.speed = SimSpeed::Fast;
        let (hours, _) = clock.tick(1.0); // 1s * 2x = 2.0s => 1 hour
        assert_eq!(hours, vec![(1, 10)]);
        assert_eq!(clock.hour, 10);
    }
}
