use bevy::prelude::*;
use std::collections::VecDeque;

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationFeed>()
            .add_message::<NotifyMessage>()
            .add_systems(Update, (collect_notifications, expire_notifications));
    }
}

#[derive(Message, Clone, Debug)]
pub struct NotifyMessage {
    pub text: String,
    pub is_warning: bool,
}

impl NotifyMessage {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_warning: false,
        }
    }

    pub fn warn(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_warning: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToastItem {
    pub text: String,
    pub is_warning: bool,
    pub ttl_secs: f32,
}

#[derive(Resource, Default, Debug)]
pub struct NotificationFeed {
    pub items: VecDeque<ToastItem>,
}

impl NotificationFeed {
    pub fn push(&mut self, text: impl Into<String>, is_warning: bool) {
        self.items.push_back(ToastItem {
            text: text.into(),
            is_warning,
            ttl_secs: 6.5,
        });
        while self.items.len() > 4 {
            self.items.pop_front();
        }
    }
}

fn collect_notifications(
    mut reader: MessageReader<NotifyMessage>,
    mut feed: ResMut<NotificationFeed>,
) {
    for msg in reader.read() {
        feed.push(msg.text.clone(), msg.is_warning);
    }
}

fn expire_notifications(time: Res<Time>, mut feed: ResMut<NotificationFeed>) {
    if feed.items.is_empty() {
        return;
    }
    let dt = time.delta_secs();
    for item in &mut feed.items {
        item.ttl_secs -= dt;
    }
    while feed.items.front().is_some_and(|item| item.ttl_secs <= 0.0) {
        feed.items.pop_front();
    }
}
