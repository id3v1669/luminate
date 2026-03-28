use iced::{Point, Rectangle, Size, advanced::svg, widget::svg::Handle};

use crate::kit::sonata::components::input::tooltip::vars::{
    SIZE_TOOLTIP_ARROW_HEIGHT, SIZE_TOOLTIP_ARROW_WIDTH, TOOLTIP_ARROW,
};

pub fn tooltip_arrow<Renderer>(renderer: &mut Renderer, position: Point, viewport: iced::Rectangle)
where
    Renderer: iced::advanced::text::Renderer<Font = iced::Font> + svg::Renderer,
{
    let handle: Handle = Handle::from_memory(TOOLTIP_ARROW.as_bytes());
    let size = Size::new(
        SIZE_TOOLTIP_ARROW_WIDTH.round(),
        SIZE_TOOLTIP_ARROW_HEIGHT.round(),
    );
    let drawing_bounds = Rectangle::new(position, size);

    renderer.draw_svg(
        svg::Svg {
            handle,
            color: None,
            rotation: 0.0.into(),
            opacity: 1.0,
        },
        drawing_bounds,
        viewport,
    );
}
