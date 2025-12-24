# Contest greeter

This is a LightDM greeter specifically created for ICPC style contests. It supports everything [lightdm-qt5-greeter-ccs-api](https://github.com/GEHACK/lightdm-qt5-greeter-ccs-api) also supports, plus some extra features.

## Features

- Wallpaper from file path or url
- Show login UI when a specific chain of characters is typed
- Countdown from n seconds to contest
- Contest start time from config or ICPC CCS contest API URL
- Instead of heavy api polling, keep track of start time internally
- Dynamically modify some greeter state via a dbus service during runtime

## Configuration

The config is located in `/etc/lightdm/lightdm-contest-greeter.conf` and is in toml format.

The syntax looks like this:

```toml
log_level = "info"
interval = 1
countdown_end_login = false
```

Possible config values: [CONFIG.md](./CONFIG.md)

## D-Bus control surface

The greeter optionally exposes a small D-Bus service that lets external tools update runtime state
(wallpaper, countdown, and session start). The module lives in `src/dbus.rs` and is enabled with
`enable_dbus = true` in the config.

Service details:
- Bus name: `nl.luukblankenstijn.ContestGreeterService`
- Object path: `/nl/luukblankenstijn/ContestGreeterService`
- Interface: `nl.luukblankenstijn.ContestGreeterService`

The supported methods are documented in `contestgreeter_interface.xml`, which is the D-Bus
introspection XML for the service.
