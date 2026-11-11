use winit::window::ResizeDirection::{self, *};

use super::*;
use crate::runtime::{Env, Runtime, TitlebarAction};
use crate::{Chrome, Cx, Titlebar, col};

const SIZE: (f32, f32) = (400.0, 300.0);

fn edge(x: f32, y: f32) -> Option<ResizeDirection> {
    resize_edge(SIZE, (x, y), true)
}

#[test]
fn the_edges_resize_from_the_side_or_corner_under_the_pointer() {
    assert_eq!(edge(200.0, 0.0), Some(North));
    assert_eq!(edge(200.0, 5.9), Some(North));
    assert_eq!(edge(200.0, 6.0), None, "the band is 6 pt");
    assert_eq!(edge(200.0, 299.0), Some(South));
    assert_eq!(edge(0.0, 150.0), Some(West));
    assert_eq!(edge(399.5, 150.0), Some(East));
    assert_eq!(edge(200.0, 150.0), None);
    // Corners take a 12 pt square, along the edges and inside them.
    assert_eq!(edge(0.0, 0.0), Some(NorthWest));
    assert_eq!(edge(11.0, 2.0), Some(NorthWest), "along the top, near the left corner");
    assert_eq!(edge(2.0, 11.0), Some(NorthWest), "along the left, near the top corner");
    assert_eq!(edge(10.0, 10.0), Some(NorthWest), "inside the corner square");
    assert_eq!(edge(13.0, 2.0), Some(North), "past the corner");
    assert_eq!(edge(395.0, 3.0), Some(NorthEast));
    assert_eq!(edge(3.0, 295.0), Some(SouthWest));
    assert_eq!(edge(390.0, 290.0), Some(SouthEast));
    assert_eq!(edge(-1.0, 150.0), None, "outside the window");
    assert_eq!(edge(400.0, 150.0), None, "outside the window");
}

#[test]
fn a_maximized_full_screen_or_fixed_window_has_no_resize_edges() {
    for (x, y) in [(0.0, 0.0), (200.0, 0.0), (399.0, 299.0), (0.0, 150.0)] {
        assert_eq!(resize_edge(SIZE, (x, y), false), None, "at {x}, {y}");
    }
}

#[test]
fn the_buttons_take_their_width_and_gaps_from_the_trailing_edge() {
    assert_eq!(width(0), 0.0);
    assert_eq!(width(1), 36.0, "close alone: 24 pt and the 12 pt margin");
    assert_eq!(width(3), 108.0);
}

struct State { pressed: u32 }

fn view(_: &State, _: &mut Cx<State>) -> El {
    col().size(SIZE.0, SIZE.1)
        .child(row().key("titlebar").size(SIZE.0, 40.0).window_drag()
            .child(el().key("tool").size(40.0, 40.0).on_press(|s: &mut State| s.pressed += 1)))
}

fn framed(titlebar: Option<Titlebar>, borderless: bool) -> Runtime<State> {
    let mut rt = Runtime::new(State { pressed: 0 }, view, SIZE, Env::new(false, 1.0));
    rt.headless_chrome(Chrome::DEFAULT, titlebar, borderless);
    rt.frame(0.0);
    rt
}

#[test]
fn a_linux_titlebar_reports_its_buttons_and_band_in_the_chrome_insets() {
    let rt = framed(Some(Titlebar::new(40.0)), false);
    assert_eq!(rt.chrome_insets(), crate::ChromeInsets::new(0.0, 108.0, 40.0));
    assert_eq!(framed(Some(Titlebar::new(40.0)), true).chrome_insets(), crate::ChromeInsets::default(), "borderless: no frame");
    assert_eq!(framed(None, false).chrome_insets(), crate::ChromeInsets::default(), "no titlebar: no frame");
}

#[test]
fn the_caption_buttons_sit_at_the_trailing_edge_centred_in_the_band() {
    let rt = framed(Some(Titlebar::new(40.0)), false);
    assert!(rt.presented(BUTTONS_KEY).is_some());
    let rects: Vec<[f32; 4]> = ["rune-caption-minimize", "rune-caption-maximize", "rune-caption-close"]
        .iter().map(|k| rt.presented(k).unwrap_or_else(|| panic!("{k} is drawn")).0).collect();
    assert_eq!(rects, [[292.0, 8.0, 24.0, 24.0], [328.0, 8.0, 24.0, 24.0], [364.0, 8.0, 24.0, 24.0]]);
    let labels: Vec<_> = rt.elements().into_iter().filter(|e| e.role == "button" && e.key.as_deref().is_some_and(|k| k.starts_with("rune-caption-")))
        .map(|e| e.label.unwrap_or_default()).collect();
    assert_eq!(labels, ["Minimize", "Maximize", "Close"]);
}

#[test]
fn the_caption_buttons_hover_and_press_as_elements_over_a_drag_region() {
    let mut rt = framed(Some(Titlebar::new(40.0)), false);
    let (x, y) = (340.0, 20.0);
    rt.pointer_move(x, y);
    rt.frame(0.2);
    let hover = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("rune-caption-maximize")).unwrap().hover;
    assert!(hover > 0.0, "the hovered button shows its disc");
    rt.pointer_down(x, y);
    assert_eq!(rt.take_titlebar_action(), None, "the button takes the press, not the drag region under it");
    rt.pointer_up(x, y);
    rt.frame(0.3);
    assert!(rt.take_toggle_maximized(), "maximize presses through normal pointer handling");
    // The drag region beside the buttons still moves the window, and its own controls press.
    rt.pointer_move(200.0, 20.0);
    rt.pointer_down(200.0, 20.0);
    assert_eq!(rt.take_titlebar_action(), Some(TitlebarAction::Drag));
    rt.pointer_up(200.0, 20.0);
    assert_eq!(rt.state.pressed, 0);
}
