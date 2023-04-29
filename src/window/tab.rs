use super::{animation::Animation, edit::Edit};

pub enum Tab {
    Edit(Edit),
    Animation(Animation),
}
