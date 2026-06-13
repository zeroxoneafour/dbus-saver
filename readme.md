## deprecated in favor of copy in polonium repo

See the updated code at https://github.com/zeroxoneafour/polonium/tree/master/dbus-saver

# dbus-saver

A daemon written in Rust, intended to be used alongside Polonium for setting management per-desktop.

## installation

Install with cargo (`cargo install --path .`) and then copy the systemd service file to `~/.config/systemd/user/`. To run on startup, do `systemctl --user enable dbus-saver`.
