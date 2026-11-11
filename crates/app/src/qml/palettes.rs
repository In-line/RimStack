use serde_json::{Map, Value};

fn theme_qml() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("qml/Theme.qml");
    std::fs::read_to_string(path).unwrap()
}

/// A palette's `key: "#…"` pairs, its high-contrast line left out.
fn palette(theme: &str) -> Map<String, Value> {
    let text = theme_qml();
    let opening = format!("        {theme}: {{\n");
    let start = text
        .find(&opening)
        .unwrap_or_else(|| panic!("no palette {theme}"))
        + opening.len();
    let block = &text[start..start + text[start..].find("\n        },").unwrap()];
    let mut found = Map::new();
    for line in block
        .lines()
        .filter(|line| !line.trim_start().starts_with("high:"))
    {
        for (at, _) in line.match_indices(": \"#") {
            let key = line[..at].rsplit([' ', '{', ',']).next().unwrap();
            let value = line[at + 3..].split('"').next().unwrap();
            found.insert(key.to_string(), Value::from(value));
        }
    }
    found
}

/// `Theme.swatches` rows: name, colour on a dark theme, colour on the light one.
fn swatches() -> Vec<(String, String, String)> {
    let text = theme_qml();
    let start = text.find("readonly property var swatches: [").unwrap();
    let block = &text[start..start + text[start..].find("\n    ]").unwrap()];
    block
        .match_indices("[\"")
        .map(|(at, _)| {
            let row = &block[at..at + block[at..].find(']').unwrap() + 1];
            let cells: Vec<String> = serde_json::from_str(row).unwrap();
            (cells[0].clone(), cells[1].clone(), cells[2].clone())
        })
        .collect()
}

fn channels(colour: &str) -> [f64; 3] {
    let hex = colour.trim_start_matches('#');
    let hex = &hex[hex.len() - 6..];
    [0, 2, 4].map(|at| f64::from(u8::from_str_radix(&hex[at..at + 2], 16).unwrap()) / 255.0)
}

fn luminance(colour: &str) -> f64 {
    let linear = |c: f64| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let [r, g, b] = channels(colour).map(linear);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// HSL hue in degrees and saturation from 0 to 1.
fn hue_and_saturation(colour: &str) -> (f64, f64) {
    let [r, g, b] = channels(colour);
    let (most, least) = (r.max(g).max(b), r.min(g).min(b));
    let spread = most - least;
    if spread == 0.0 {
        return (0.0, 0.0);
    }
    let lightness = (most + least) / 2.0;
    let saturation = spread / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue = if most == r {
        60.0 * ((g - b) / spread).rem_euclid(6.0)
    } else if most == g {
        60.0 * ((b - r) / spread + 2.0)
    } else {
        60.0 * ((r - g) / spread + 4.0)
    };
    (hue, saturation)
}

fn colour<'a>(palette: &'a Map<String, Value>, key: &str) -> &'a str {
    palette
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("no {key}"))
}

const THEMES: [&str; 4] = ["rim", "dark", "oled", "light"];

#[test]
fn every_built_in_theme_passes_the_contrast_pairs() {
    for theme in THEMES {
        let short = rimstack_core::kept::owntheme::short_pairs(&palette(theme));
        assert!(short.is_empty(), "{theme}: {short:?}");
    }
}

#[test]
fn side_panes_sit_darker_and_cards_lighter_than_the_window() {
    for theme in THEMES {
        let p = palette(theme);
        let window = luminance(colour(&p, "win"));
        assert!(luminance(colour(&p, "panel")) < window, "{theme} panel");
        assert!(luminance(colour(&p, "card")) > window, "{theme} card");
    }
}

#[test]
fn no_category_looks_like_a_warning_or_an_error() {
    // Warm hues are for warnings and errors; a grey is allowed.
    for (name, dark, light) in swatches().into_iter().take(9) {
        for colour in [dark, light] {
            let (hue, saturation) = hue_and_saturation(&colour);
            assert!(
                saturation < 0.2 || !(0.0..=60.0).contains(&hue),
                "{name} {colour}"
            );
        }
    }
}

#[test]
fn colours_beyond_the_categories_stay_offered() {
    let names: Vec<String> = swatches().into_iter().map(|(name, ..)| name).collect();
    for kept in ["Amber", "Coral", "Sand", "Orange"] {
        assert!(names[9..].contains(&kept.to_string()), "{kept}");
    }
}

#[test]
fn rim_is_graphite_and_periwinkle() {
    let rim = palette("rim");
    assert_eq!(colour(&rim, "accent"), "#5466c8");
    assert_eq!(colour(&rim, "win"), "#1c1b19");
}

#[test]
fn a_palette_is_read_from_the_theme() {
    let light = palette("light");
    assert_eq!(colour(&light, "ink"), "#1d1d1f");
    assert!(!light.contains_key("onSelected"));
    assert_eq!(swatches()[0].0, "Violet");
}
