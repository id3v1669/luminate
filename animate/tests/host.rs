//! The host is the integration point every user runs: it must tick once per
//! `RedrawRequested` and nowhere else, through iced's real event path.

use std::time::Duration;

use iced::time::Instant;
use iced::{Element, Event, Settings, Size, window};
use iced_animate::widget::shape;
use iced_animate::{Motion, curves::QUICK, key};
use iced_test::Simulator;

fn redraw(at: Instant) -> Event {
    Event::Window(window::Event::RedrawRequested(at))
}

#[test]
fn a_host_advances_the_engine_once_per_redraw_event() {
    let motion = Motion::new();
    let key = key!();
    let _ = motion.to(key, QUICK, 40.0_f32);
    let width = motion.to(key, QUICK, 200.0_f32);

    let root: Element<'_, ()> = motion.host(shape().width(&width).height(40.0)).into();
    let mut ui: Simulator<'_, ()> =
        Simulator::with_size(Settings::default(), Size::new(400.0, 200.0), root);

    let start = Instant::now();
    let _ = ui.simulate([redraw(start)]);
    assert_eq!(width.get(), 40.0, "the first frame starts the clock");

    let _ = ui.simulate([redraw(start + Duration::from_millis(16))]);
    let after_one = width.get();
    assert!(
        after_one > 40.0 && after_one < 200.0,
        "one frame in: {after_one}"
    );

    let _ = ui.simulate([redraw(start + Duration::from_millis(16))]);
    assert_eq!(
        width.get(),
        after_one,
        "a repeated timestamp advances nothing"
    );

    let _ = ui.simulate([iced::Event::Mouse(iced::mouse::Event::CursorLeft)]);
    assert_eq!(width.get(), after_one, "only redraws advance the clock");

    for frame in 2..=200_u64 {
        let _ = ui.simulate([redraw(start + Duration::from_millis(16 * frame))]);
    }
    assert_eq!(width.get(), 200.0);
    assert!(!width.is_animating());
}

// ---------------------------------------------------------------------------
// The host through `iced_runtime::UserInterface`, frame by frame: it must
// keep asking for frames while something moves, stop when everything is
// settled, relayout only for `Layout`-tier tracks and collect settled tracks
// across rebuilds.

mod frame_loop {
    use std::time::Duration;

    use iced::advanced::clipboard;
    use iced::advanced::renderer::Headless;
    use iced::advanced::widget::{Operation, operation};
    use iced::time::Instant;
    use iced::widget::container;
    use iced::{Color, Event, Font, Pixels, Size, mouse, window};
    use iced_animate::widget::shape;
    use iced_animate::{Curve, Motion, MotionKey, SpringParams, key};
    use iced_test::Selector;
    use iced_test::runtime::user_interface::{self, State, UserInterface};

    const FAST: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_millis(300)));
    const SIZE: Size = Size::new(300.0, 200.0);
    const FRAME: Duration = Duration::from_millis(16);

    type Ui<'a> = UserInterface<'a, (), iced::Theme, iced::Renderer>;

    fn renderer() -> iced::Renderer {
        iced_test::futures::futures::executor::block_on(<iced::Renderer as Headless>::new(
            Font::DEFAULT,
            Pixels(16.0),
            Some("tiny-skia"),
        ))
        .expect("tiny_skia needs no GPU")
    }

    /// A box whose width follows `width_key` and whose colour follows `fill_key`.
    fn view(
        motion: &Motion,
        width_key: MotionKey,
        fill_key: MotionKey,
        width: f32,
        fill: Color,
    ) -> iced::Element<'_, ()> {
        let side = motion.to(width_key, FAST, width);
        let colour = motion.to(fill_key, FAST, fill);
        motion
            .host(container(shape().width(side).height(20.0).fill(colour)).id("box"))
            .into()
    }

    fn build<'a>(
        element: iced::Element<'a, ()>,
        cache: user_interface::Cache,
        renderer: &mut iced::Renderer,
    ) -> Ui<'a> {
        UserInterface::build(element, SIZE, cache, renderer)
    }

    /// Runs one `RedrawRequested` and reports what the interface asked for.
    fn redraw(
        ui: &mut Ui<'_>,
        renderer: &mut iced::Renderer,
        now: Instant,
    ) -> (window::RedrawRequest, bool) {
        let mut messages = Vec::new();
        let (state, _) = ui.update(
            &[Event::Window(window::Event::RedrawRequested(now))],
            mouse::Cursor::Unavailable,
            renderer,
            &mut clipboard::Null,
            &mut messages,
        );
        match state {
            State::Updated {
                redraw_request,
                has_layout_changed,
                ..
            } => (redraw_request, has_layout_changed),
            State::Outdated => panic!("nothing here invalidates widgets"),
        }
    }

    fn box_width(ui: &mut Ui<'_>, renderer: &iced::Renderer) -> f32 {
        let mut find = iced_test::selector::id("box").find();
        ui.operate(renderer, &mut operation::black_box(&mut find));
        match find.finish() {
            operation::Outcome::Some(Some(target)) => target.bounds().width,
            _ => panic!("the box is on screen"),
        }
    }

    #[test]
    fn the_host_asks_for_frames_until_settled_and_relayouts_only_while_the_width_moves() {
        let motion = Motion::new();
        let width_key = key!();
        let fill_key = key!();
        let mut renderer = renderer();

        // First sight: 40 wide, black, nothing animates yet.
        let first = build(
            view(&motion, width_key, fill_key, 40.0, Color::BLACK),
            user_interface::Cache::default(),
            &mut renderer,
        );
        let mut ui = build(
            view(&motion, width_key, fill_key, 120.0, Color::BLACK),
            first.into_cache(),
            &mut renderer,
        );

        let mut now = Instant::now();
        // The first frame only starts the engine's clock.
        let _ = redraw(&mut ui, &mut renderer, now);
        let mut requests = Vec::new();
        let mut relayouts = Vec::new();
        let mut widths = Vec::new();
        for _ in 0..90 {
            now += FRAME;
            let (request, relayout) = redraw(&mut ui, &mut renderer, now);
            requests.push(request);
            relayouts.push(relayout);
            widths.push(box_width(&mut ui, &renderer));
        }

        assert_eq!(
            requests[0],
            window::RedrawRequest::NextFrame,
            "moving: the host asks for the next frame"
        );
        let settled_at = requests
            .iter()
            .position(|r| *r == window::RedrawRequest::Wait)
            .expect("a 300 ms spring settles well inside 90 frames");
        assert!(
            settled_at > 3,
            "settled suspiciously early at frame {settled_at}"
        );
        assert!(
            requests[settled_at..]
                .iter()
                .all(|r| *r == window::RedrawRequest::Wait),
            "once settled the host must stay quiet: {requests:?}"
        );
        assert!(
            widths[0] > 40.0 && widths[0] < 120.0,
            "frame 1 is mid-flight: {}",
            widths[0]
        );
        assert!(
            (widths[89] - 120.0).abs() < 0.5,
            "the final width is the target: {}",
            widths[89]
        );
        assert!(relayouts[0], "a moving width costs a relayout");
        assert!(
            !relayouts[settled_at..].iter().any(|r| *r),
            "no relayout after settling: {relayouts:?}"
        );
    }

    #[test]
    fn a_paint_only_animation_never_relayouts() {
        let motion = Motion::new();
        let width_key = key!();
        let fill_key = key!();
        let mut renderer = renderer();

        let first = build(
            view(&motion, width_key, fill_key, 40.0, Color::BLACK),
            user_interface::Cache::default(),
            &mut renderer,
        );
        let mut ui = build(
            view(&motion, width_key, fill_key, 40.0, Color::WHITE),
            first.into_cache(),
            &mut renderer,
        );

        let mut now = Instant::now();
        let _ = redraw(&mut ui, &mut renderer, now);
        now += FRAME;
        let (request, relayout) = redraw(&mut ui, &mut renderer, now);
        assert_eq!(
            request,
            window::RedrawRequest::NextFrame,
            "the colour is moving"
        );
        assert!(!relayout, "a colour is read in draw, never in layout");
        for _ in 0..60 {
            now += FRAME;
            let (_, relayout) = redraw(&mut ui, &mut renderer, now);
            assert!(!relayout);
        }
    }

    #[test]
    fn settled_tracks_are_collected_across_rebuilds() {
        let motion = Motion::new();
        let width_key = key!();
        let fill_key = key!();
        let mut renderer = renderer();

        let mut cache = user_interface::Cache::default();
        let mut now = Instant::now();
        // Each rebuild retargets; between rebuilds the spring settles.
        for round in 0..6_u8 {
            let width = if round % 2 == 0 { 40.0 } else { 120.0 };
            let mut ui = build(
                view(&motion, width_key, fill_key, width, Color::BLACK),
                cache,
                &mut renderer,
            );
            for _ in 0..60 {
                now += FRAME;
                let _ = redraw(&mut ui, &mut renderer, now);
            }
            cache = ui.into_cache();
        }
        assert!(
            motion.track_count() <= 2,
            "two keys are ever declared, so at most two tracks may live: {}",
            motion.track_count()
        );
    }
}
