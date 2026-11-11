- **`App::titlebar` replaces the system title bar on Linux.** On X11 and Wayland a window with a
  titlebar now opens without decorations, and the app's band is its title bar, as on Windows:
  Rune draws minimize, maximize/restore and close at the trailing edge as round 24 pt buttons in
  the GNOME and KDE style, `Cx::chrome_insets` reports their width and the band's height, and
  headless runs draw them too. `.window_drag()` regions move the window and maximize on a
  double click; a right click on one opens the compositor's window menu (Wayland). A resizable,
  restored window gets invisible resize edges, 6 pt along the sides and 12 pt squares at the
  corners, with resize cursors. Full screen hides the buttons and the edges. The window has no
  system shadow or border. The buttons' symbols follow the system's light or dark setting, as on
  Windows: an app whose theme doesn't follow it sets `Cx::caption_color`, or a light theme on a
  dark desktop gets white buttons on a white bar.
