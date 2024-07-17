# Event Command

I wrote `ev-cmd` because I didn't have an easy way to bind commands to my [Koolertron macropad](https://www.koolertron.com/koolertron-single-handed-programmable-mechanical-keyboard-with-48-programmable-keys.html). 

## Install

Install from the AUR using

```
yay -S ev-dev
```

Build using `cargo build --release`

## Usage

Get the device file from `/dev/input/by-id`

```
Usage: ev-cmd [OPTIONS] --device-path <DEVICE_PATH>

Options:
  -d, --device-path <DEVICE_PATH>  
  -c, --config-path <CONFIG_PATH>  
  -h, --help                       Print help
```
