//! The client-side frame of a Linux window with `App::titlebar` (X11 and Wayland): the caption
//! buttons, styled as GNOME and KDE draw theirs, and the invisible resize edges around the
//! window.
//!
//! The window has no decorations, so the system draws nothing: Rune draws round 24 pt buttons
//! 12 pt apart and 12 pt from the trailing edge, centred in the band, each a thin symbol with a
//! faint disc of the symbol colour under the pointer. They are ordinary elements: the pointer
//! hovers and presses them as any other, and their `on_press` minimizes, maximizes or restores,
//! and closes. The resize edges are a 6 pt band around a restored, resizable window, widening
//! to 12 pt squares at the corners; the host takes the pointer there before the view does.
#![cfg_attr(not(all(unix, not(target_os = "macos"), not(target_os = "ios"), not(target_os = "android"))), allow(dead_code))]

use winit::window::ResizeDirection;

use crate::el::{el, row};
use crate::{Color, El, LineCap, Shape};

use super::caption::{BUTTONS_KEY, action, identity, quad, restore, rounded};
use super::{Caption, CaptionButton};

/// A button's diameter, in points.
pub(crate) const BUTTON: f32 = 24.0;
/// Between two buttons, and from the last one to the trailing edge, in points.
pub(crate) const GAP: f32 = 12.0;
/// The resize band along each edge, in points.
pub(crate) const EDGE: f32 = 6.0;
/// The side of the square at each corner that resizes both ways, in points.
pub(crate) const CORNER: f32 = 12.0;

/// The disc under a hovered button, and the extra one under a pressed button, as fractions of
/// the symbol colour's alpha.
const HOVER_ALPHA: f32 = 0.10;
const PRESSED_ALPHA: f32 = 0.12;
/// A disabled button's symbol, as a fraction of the symbol colour's alpha.
const DISABLED_ALPHA: f32 = 0.36;
/// The symbol's side, in points.
const GLYPH: f32 = 8.0;

/// Points `count` buttons take from the trailing edge: the buttons, the gaps between them and
/// the margin after the last.
pub(crate) fn width(count: usize) -> f32 {
    if count == 0 { return 0.0; }
    let n = count as f32;
    n * BUTTON + n * GAP
}

/// The buttons in a row at the frame's top-right corner, the band's height, over everything the
/// view drew. `scale` is device pixels per view point and `zoom` the view's `Cx::zoom`: the
/// buttons keep the window's size at every zoom.
pub(super) fn buttons(caption: &Caption, symbol: Color, scale: f32, zoom: f32) -> El {
    let zoom = if zoom > 0.0 { zoom } else { 1.0 };
    let window = scale / zoom;
    row().key(BUTTONS_KEY).pin(None, Some(0.0), Some(0.0), None).h(caption.height / zoom).items_center().gap(GAP / zoom).pr(GAP / zoom)
        .children(caption.buttons().iter().map(|&button| caption_button(caption, button, symbol, (scale, window), zoom)))
}

fn caption_button(caption: &Caption, button: CaptionButton, symbol: Color, (scale, window): (f32, f32), zoom: f32) -> El {
    let enabled = caption.enabled(button);
    let ink = if enabled { symbol } else { symbol.alpha(symbol.a * DISABLED_ALPHA) };
    let (key, label) = identity(button, caption.maximized);
    let mut e = el().key(key).role("button").label(label).focusable(false).disabled(!enabled)
        .size(BUTTON / zoom, BUTTON / zoom).radius(BUTTON / zoom / 2.0)
        .child(glyph(button, caption.maximized, ink, (scale, window)));
    if enabled {
        e = e.hover(symbol.alpha(symbol.a * HOVER_ALPHA)).press(symbol.alpha(symbol.a * PRESSED_ALPHA));
        e.handlers.on_press = Some(action(button));
    }
    e
}

/// The symbol, laid out in device pixels so its strokes land on whole pixels: `scale` is device
/// pixels per view point, `window` per window point, which sizes the glyph.
fn glyph(button: CaptionButton, maximized: bool, ink: Color, (scale, window): (f32, f32)) -> El {
    let g = (GLYPH * window).round();
    let s = window.round().max(1.0);
    let side = (BUTTON * window).round();
    let at = ((side - g) / 2.0).floor();
    let pt = 1.0 / scale;
    let shape = match button {
        CaptionButton::Minimize => Shape::new().fill(quad(0.0, g - s, g, s, pt), ink),
        CaptionButton::Maximize if maximized => restore(g, s, (scale, window), ink),
        CaptionButton::Maximize => {
            let r = (1.0 * window).floor();
            Shape::new().stroke_with(rounded([s / 2.0, s / 2.0, g - s / 2.0, g - s / 2.0], [r; 4], pt), true, s * pt, LineCap::Round, ink)
        }
        CaptionButton::Close => {
            let p = |x: f32, y: f32| [x * pt, y * pt];
            Shape::new()
                .line(p(0.0, 0.0), p(g, g), s * pt * 1.1, LineCap::Round, ink)
                .line(p(g, 0.0), p(0.0, g), s * pt * 1.1, LineCap::Round, ink)
        }
    };
    el().absolute(at * pt, at * pt).size(g * pt, g * pt).shape(shape).a11y_hidden()
}

/// The edge or corner a press at `(x, y)` resizes a window `size` points big from, in points
/// from its top-left: within [`EDGE`] of a side, or within the [`CORNER`] square at a corner.
/// None when the window can't be resized from its edges (`resizable` false: not resizable,
/// maximized or full screen) or the point is inside.
pub(crate) fn resize_edge((w, h): (f32, f32), (x, y): (f32, f32), resizable: bool) -> Option<ResizeDirection> {
    if !resizable || x < 0.0 || y < 0.0 || x >= w || y >= h { return None; }
    let (left, right, top, bottom) = (x < CORNER, x >= w - CORNER, y < CORNER, y >= h - CORNER);
    let on_edge = x < EDGE || y < EDGE || x >= w - EDGE || y >= h - EDGE;
    let in_corner = (left || right) && (top || bottom);
    if !on_edge && !in_corner { return None; }
    Some(match (top, bottom, left, right) {
        (true, _, true, _) => ResizeDirection::NorthWest,
        (true, _, _, true) => ResizeDirection::NorthEast,
        (_, true, true, _) => ResizeDirection::SouthWest,
        (_, true, _, true) => ResizeDirection::SouthEast,
        _ if y < EDGE => ResizeDirection::North,
        _ if y >= h - EDGE => ResizeDirection::South,
        _ if x < EDGE => ResizeDirection::West,
        _ => ResizeDirection::East,
    })
}

#[cfg(all(test, unix, not(target_os = "macos"), not(target_os = "ios"), not(target_os = "android")))]
#[path = "csd_tests.rs"]
mod tests;
