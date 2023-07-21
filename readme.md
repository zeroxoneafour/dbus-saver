# dbus-saver

A daemon written in Rust, intended to be used alongside Polonium for setting management per-desktop.

## installation

Install with cargo (`cargo install --path .`) and then copy the systemd service file to `~/.config/systemd/user/`. To run on startup, do `systemctl --user enable dbus-saver`.
