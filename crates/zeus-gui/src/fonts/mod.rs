use eframe::egui::{epaint::text::FontDefinitions, FontData, FontFamily, FontTweak};
use std::borrow::Cow;


pub fn get_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    let roboto_regular = include_bytes!("./roboto/Roboto-Regular.ttf");
    let roboto_bold = include_bytes!("./roboto/Roboto-Bold.ttf");

    let roboto_regular_font = FontData {
        font: Cow::Borrowed(roboto_regular),
        index: 0,
        tweak: FontTweak::default()
    };

    let roboto_bold_font = FontData {
        font: Cow::Borrowed(roboto_bold),
        index: 0,
        tweak: FontTweak::default()
    };

    add_font(&mut fonts, roboto_regular_font, "Roboto-Regular");
    add_font(&mut fonts, roboto_bold_font, "Roboto-Bold");

    fonts
}

/// Returns Roboto-Regular font
pub fn roboto_regular() -> FontFamily {
    FontFamily::Name("Roboto-Regular".into())
}

/// Returns Roboto-Bold font
pub fn roboto_bold() -> FontFamily {
    FontFamily::Name("Roboto-Bold".into())
}

fn add_font(font: &mut FontDefinitions, font_data: FontData, font_name: &str) {
    font.font_data.insert(font_name.into(), font_data);

    font.families.insert(FontFamily::Name(font_name.into()), vec![font_name.into()]);
}