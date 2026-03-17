use iced::{Color, Padding, border::Radius};

pub const fn padding_vh(v: f32, h: f32) -> Padding {
    Padding {
        top: v,
        right: h,
        bottom: v,
        left: h,
    }
}

pub const fn radius_all(r: f32) -> Radius {
    Radius {
        top_left: r,
        top_right: r,
        bottom_right: r,
        bottom_left: r,
    }
}

/// Linear interpolation between two colors.
pub const fn mix(base: Color, overlay: Color, factor: f32) -> Color {
    Color {
        r: base.r + (overlay.r - base.r) * factor,
        g: base.g + (overlay.g - base.g) * factor,
        b: base.b + (overlay.b - base.b) * factor,
        a: base.a + (overlay.a - base.a) * factor,
    }
}

pub const fn lighten(color: Color, amount: f32) -> Color {
    mix(color, Color::WHITE, amount)
}

pub const fn darken(color: Color, amount: f32) -> Color {
    mix(color, Color::BLACK, amount)
}

pub const fn with_alpha(color: Color, alpha: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: alpha,
    }
}

/// Compose `foreground` over `background`
pub const fn alpha_blend(background: Color, foreground: Color) -> Color {
    let fa = foreground.a;
    let ba = background.a;
    let out_a = fa + ba * (1.0 - fa);

    if out_a == 0.0 {
        return Color::TRANSPARENT;
    }

    Color {
        r: (foreground.r * fa + background.r * ba * (1.0 - fa)) / out_a,
        g: (foreground.g * fa + background.g * ba * (1.0 - fa)) / out_a,
        b: (foreground.b * fa + background.b * ba * (1.0 - fa)) / out_a,
        a: out_a,
    }
}
