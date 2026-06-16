mod about;
mod create;
mod dashboard;
mod home;

pub use about::AboutPage;
pub use create::CreatePage;
pub use dashboard::DashboardPage;
pub use home::HomePage;

use std::hash::{DefaultHasher, Hash, Hasher};

pub fn name_to_hex_color(name: &str) -> String {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let hash_value = hasher.finish();

    let hue = (hash_value % 360) as f32;

    let saturation = 0.65;
    let lightness = 0.45;

    let (r, g, b) = hsl_to_rgb(hue, saturation, lightness);

    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r_p, g_p, b_p) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r_p + m) * 255.0).round() as u8,
        ((g_p + m) * 255.0).round() as u8,
        ((b_p + m) * 255.0).round() as u8,
    )
}
