use crate::{
    app::{HEIGHT, WIDTH},
    icons::IconTextures,
};
use eframe::egui::{
     style::{Selection, WidgetVisuals, Widgets}, vec2, Color32, Context, Frame, Margin, Mesh, Pos2, Rect, Rounding, Stroke, Visuals,
};
use gradient::vertical_gradient_mesh_2;
use std::sync::Arc;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref THEME: ZeusTheme = ZeusTheme::default();
}

pub mod gradient;

// credits: https://github.com/4JX/mCubed/blob/master/main/src/ui/app_theme.rs
/// Holds the Theme Settings for the whole App
pub struct ZeusTheme {
    pub colors: Colors,

    pub visuals: Visuals,

    pub rounding: RoundingTypes,

    pub default_panel_frame: Frame,

    pub prompt_frame: Frame,

    pub icons: Arc<IconTextures>,

    pub bg_gradient: Mesh,
}

impl ZeusTheme {
    pub fn new(ctx: &Context) -> Self {
        let colors = Colors::default();
        let icons = Arc::new(IconTextures::new(ctx).unwrap());

        let rect = Rect::from_min_size(Pos2 { x: 0.0, y: 0.0 }, vec2(WIDTH, HEIGHT));

        let top_color = Color32::from_rgb(44,4,67); // Dark Purple
        let mid_color = Color32::from_rgb(145,60,167); // Mid Pink
        let bottom_color = Color32::from_rgb(0,0,0); // Black

         // make a gradient from a Color32
        /* 
        let t = remap(0.0, rect.x_range(), -1.0..=1.0).abs();
        let int_top_color = Color32::from_rgb(
            lerp(top_color.r() as f32..=bottom_color.r() as f32, t) as u8,
            lerp(top_color.g() as f32..=bottom_color.g() as f32, t) as u8,
            lerp(top_color.b() as f32..=bottom_color.b() as f32, t) as u8,
        );
        */

        let mesh_colors = [
        top_color,  
        mid_color,  
        bottom_color,
        ];

        // Create the gradient mesh
        let mesh = vertical_gradient_mesh_2(rect, &mesh_colors);

        let widgets = Widgets {
            noninteractive: WidgetVisuals {
                bg_fill: Color32::TRANSPARENT, // window background color
                weak_bg_fill: Color32::TRANSPARENT,
                bg_stroke: Stroke::new(0.5, Color32::TRANSPARENT), // separators, indentation lines, windows outlines
                fg_stroke: Stroke::new(1.0, Color32::from_gray(140)), // normal text color
                rounding: Rounding::same(2.0),
                expansion: 0.0,
            },
            // Affects the visuals of widgets like buttons, comboboxes, etc.
            // When they are not hovered or clicked.
            inactive: WidgetVisuals {
                bg_fill: Color32::WHITE, // button background
                weak_bg_fill: Color32::TRANSPARENT,
                bg_stroke: Stroke::new(0.5, Color32::WHITE),
                fg_stroke: Stroke::new(1.0, Color32::WHITE),
                rounding: Rounding::same(5.0),
                expansion: 0.0,
            },
            // When the widget is hovered
            hovered: WidgetVisuals {
                bg_fill: Color32::WHITE,
                weak_bg_fill: Color32::TRANSPARENT,
                bg_stroke: Stroke::new(1.0, Color32::WHITE), // e.g. hover over window edge or button
                fg_stroke: Stroke::new(1.5, Color32::WHITE),
                rounding: Rounding::same(5.0),
                expansion: 1.0,
            },
            // When the widget is clicked
            active: WidgetVisuals {
                bg_fill: Color32::TRANSPARENT,
                weak_bg_fill: colors.silver, // affects bg color of widgets like buttons when clicked
                bg_stroke: Stroke::new(1.0, Color32::WHITE),
                fg_stroke: Stroke::new(2.0, Color32::WHITE),
                rounding: Rounding::same(5.0),
                expansion: 1.0,
            },
            open: WidgetVisuals {
                bg_fill: Color32::TRANSPARENT,
                weak_bg_fill: Color32::TRANSPARENT,
                bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
                fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
                rounding: Rounding::same(2.0),
                expansion: 0.0,
            },
        };

        let selection = Selection {
            bg_fill: colors.light_gray,
            ..Selection::default()
        };

        let visuals = Visuals {
            dark_mode: false,
            override_text_color: Some(colors.white),
            widgets,
            selection,
            extreme_bg_color: Color32::TRANSPARENT, // affects the bg color of widgets like [TextEdit]
            // panel_fill: Color32::TRANSPARENT,
            ..Visuals::default()
        };

        let default_panel_frame = Frame {
            inner_margin: Margin::same(8.0),
            fill: Color32::TRANSPARENT,
            ..Frame::default()
        };

        let rounding = RoundingTypes::default();
        let prompt_frame = default_panel_frame.rounding(rounding.big);

        Self {
            colors,
            visuals,
            default_panel_frame,
            prompt_frame,
            rounding,
            icons,
            bg_gradient: mesh,
        }
    }
}

impl Default for ZeusTheme {
    fn default() -> Self {
        let ctx = Context::default();
        Self::new(&ctx)
    }
}

pub struct RoundingTypes {
    pub small: Rounding,
    pub big: Rounding,
}

impl Default for RoundingTypes {
    fn default() -> Self {
        Self {
            small: Rounding::same(2.0),
            big: Rounding::same(4.0),
        }
    }
}

pub struct Colors {
    pub white: Color32,
    pub silver: Color32,
    pub gray: Color32,
    pub dark_gray: Color32,
    pub darker_gray: Color32,
    pub light_gray: Color32,
    pub lighter_gray: Color32,
    pub error_message: Color32,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            white: Color32::from_rgb(255, 255, 255),
            silver: Color32::from_rgb(192, 192, 192),
            gray: Color32::from_rgb(58, 58, 58),
            dark_gray: Color32::from_rgb(38, 38, 38),
            darker_gray: Color32::from_rgb(22, 22, 22),
            light_gray: Color32::from_rgb(85, 85, 85),
            lighter_gray: Color32::from_rgb(120, 120, 120),
            error_message: Color32::from_rgb(211, 80, 80),
        }
    }
}
