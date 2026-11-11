//! The Linux client-side frame of a window with `App::titlebar` (`titlebar::csd`): the window
//! has no decorations, so its resize edges are Rune's. The pointer over an edge shows the
//! matching resize cursor and leaves the view; a press there hands the resize to the window
//! manager or compositor (`drag_resize_window`).

use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::window::{CursorIcon, ResizeDirection, Window};

use crate::runtime::Runtime;
use super::Win;

/// A Linux window's own frame: present while it has a titlebar.
#[derive(Default)]
pub(super) struct ClientFrame {
    /// The edge under the pointer, whose resize cursor shows.
    edge: Option<ResizeDirection>,
    /// A press the view took is still held: its drag goes on into the edges.
    view_press: bool,
}

impl ClientFrame {
    /// The window manager took the press the view had (`drag_window`), and may keep its
    /// release: the edges answer again.
    pub(super) fn press_taken(&mut self) {
        self.view_press = false;
    }
}

/// Takes the pointer events the window's resize edges own before the view sees them: a move
/// onto an edge (the view sees the pointer leave) and a press there. True when it took `event`.
pub(super) fn edge_event<S: 'static>(l: &mut Win, rt: &mut Runtime<S>, event: &WindowEvent, scale: f32) -> bool {
    let Some(frame) = l.native.frame.as_mut() else { return false };
    match *event {
        WindowEvent::CursorMoved { position, .. } => {
            let at = (position.x as f32 / scale, position.y as f32 / scale);
            let edge = if frame.view_press { None } else { edge_at(&l.window, at, scale) };
            let was = std::mem::replace(&mut frame.edge, edge);
            let Some(edge) = edge else { return false };
            if was != Some(edge) {
                if was.is_none() {
                    rt.pointer_leave();
                    // The view's cursor gives way (the arrow, once) before the edge's shows.
                    let focused = l.window.has_focus();
                    l.apply_cursor(rt, focused);
                }
                l.window.set_cursor(cursor(edge));
            }
            true
        }
        WindowEvent::CursorLeft { .. } => {
            frame.edge = None;
            false
        }
        WindowEvent::MouseInput { state, button, .. } => match (frame.edge, state) {
            (Some(edge), ElementState::Pressed) => {
                if button == MouseButton::Left { let _ = l.window.drag_resize_window(edge); }
                true
            }
            (Some(_), ElementState::Released) => true,
            (None, state) => {
                frame.view_press = state == ElementState::Pressed;
                false
            }
        },
        _ => false,
    }
}

/// The edge a press at `at` (points) would resize the window from: none while it is maximized,
/// full screen or not resizable.
fn edge_at(window: &Window, at: (f32, f32), scale: f32) -> Option<ResizeDirection> {
    let size = window.inner_size();
    let resizable = window.is_resizable() && !window.is_maximized() && window.fullscreen().is_none();
    crate::titlebar::resize_edge((size.width as f32 / scale, size.height as f32 / scale), at, resizable)
}

/// The cursor over `edge`.
fn cursor(edge: ResizeDirection) -> CursorIcon {
    match edge {
        ResizeDirection::North => CursorIcon::NResize,
        ResizeDirection::South => CursorIcon::SResize,
        ResizeDirection::East => CursorIcon::EResize,
        ResizeDirection::West => CursorIcon::WResize,
        ResizeDirection::NorthEast => CursorIcon::NeResize,
        ResizeDirection::NorthWest => CursorIcon::NwResize,
        ResizeDirection::SouthEast => CursorIcon::SeResize,
        ResizeDirection::SouthWest => CursorIcon::SwResize,
    }
}
