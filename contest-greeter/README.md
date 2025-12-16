# Contest greeter

This is a LightDM greeter specifically created for ICPC style contests. It supports everything [lightdm-qt5-greeter-ccs-api](https://github.com/GEHACK/lightdm-qt5-greeter-ccs-api) also supports, plus some extra features.

### Features

- Wallpaper from file path or url
- Show login UI when a specific chain of characters is typed
- Countdown from n seconds to contest
- Contest start time from config or ICPC CCS contest API URL
- Instead of heavy api polling, keep track of start time internally
- (planned) Dynamically modify wallpaper image during runtime via a socket

### Configuration

The config is located in `/etc/lightdm/lightdm-contest-greeter.conf` and is in toml format.

The syntax looks like this:

```toml
log_level = "info"
interval = 1
countdown_end_login = false
```

Possible config values: [CONFIG.md](./CONFIG.md)
