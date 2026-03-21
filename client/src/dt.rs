use chrono::{TimeDelta};

pub fn format_fuzzy_short(delta: TimeDelta) -> String {
    let days = delta.num_days();
    let hours = delta.num_hours();
    let minutes = delta.num_minutes();

    if days >= 3 {
        format!("{}d", days)
    } else if days >= 1 {
        let remaining_hours = hours % 24;
        format!("{}d {}h", days, remaining_hours)
    } else if hours >= 1 {
        format!("{}h", hours)
    } else {
        format!("{}m", minutes)
    }
}

pub fn format_fuzzy_dist(dt: chrono::DateTime<chrono::FixedOffset>) -> String {
    let now = chrono::Utc::now();
    let delta = now.signed_duration_since(dt);
    format_fuzzy_short(delta)
}