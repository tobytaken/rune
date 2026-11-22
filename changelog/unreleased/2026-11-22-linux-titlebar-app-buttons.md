- **An app can draw the window's buttons itself on Linux.** `Titlebar::without_buttons()` keeps
  the client-side frame (resize edges, `.window_drag()` regions, the window menu) but draws no
  caption buttons, and `Cx::chrome_insets` takes no room for them: the app puts its own where
  the desktop's button layout says, as GTK's header bars do, and presses them with
  `Tx::minimize`, `Tx::toggle_maximized` and `Tx::close_window`. `ChromeInsets` now says whether
  the window is `maximized` or `full_screen`, for the app's own Maximize or Restore, and the view
  re-runs when either changes. A headless run with `App::titlebar` keeps what
  `Tx::toggle_maximized` and `Tx::toggle_full_screen` asked for, so tests see the state.
