# Logitech G600 Driver (Rust)

This is a Rust driver for the Logitech G600 mouse that lets you create custom button mappings and macros. You can set up different profiles for various games, trigger commands when buttons are pressed or released, and even run shell scripts when switching between profiles. It's designed to make the G600 more customizable for gaming scenarios where you need quick access to specific commands.

## Installation

### Option 1: Using Nix Flakes (Recommended)

If you have Nix with flakes enabled, you can install directly:

```bash
# Install to your profile
nix profile install github:danhab99/logitech-g600-rs

# Or run directly without installing
nix run github:danhab99/logitech-g600-rs -- /dev/input/mouse0

# Or build locally
nix build
./result/bin/logitech-g600-rs /dev/input/mouse0
```

### Option 2: Building from Source

1. **Clone the Repository:**
    ```bash
    git clone https://github.com/danhab99/logitech-g600-rs.git
    cd logitech-g600-rs
    ```

2. **Build the Project:**
    ```bash
    cargo build --release
    ```
3. **Setup your mouse:**
    ```bash
    make setup
    ```
    > **Warning, this will reset all of your mouse settings**

4. **Run the Executable:** 
    You'll need to run the executable with the device path as an argument. The device path is usually `/dev/input/mouse0` (check this using `ls /dev/input/by-id`).
    ```bash
    ./target/release/g600-rs /dev/input/mouse0
    ```

## Usage

```
Usage: logitech-g600-rs [OPTIONS] --device-path <DEVICE_PATH>

Options:
  -d, --device-path <DEVICE_PATH>  
  -c, --config-path <CONFIG_PATH>  
  -x, --debug                      
  -l, --lock-file <LOCK_FILE>      
  -h, --help                       Print help
```

## Configuration

The system utilizes a TOML configuration file `g600.toml` to define the macros. The configuration is structured around individual profiles, each treated as a distinct header within the TOML file. Inside each profile header, you can bind specific commands to be executed on defined keys.

### Key Points:

- **Profile Headers:** Each section in the TOML file (e.g., `[tf2]`, `[overwatch]`, `[finals]`) represents a separate profile.
- **Button Naming:** Buttons are identified using the `G` followed by a number (9-20). For example, `G9`, `G13`, etc.
- **Directional Keys:** The `UP` or `DOWN` suffix designates whether a command should be executed on an _up_ or _down_ step (e.g., `G9_DOWN`, `G9_UP`).
- **Modifier Key:** The `_MOD` suffix indicates that the macro should be triggered when the modifier button (typically the keypad G button) is _also_ pressed along with the designated button.
- **ON_ACTIVATE:** This field specifies a shell command that is executed when you switch to the corresponding profile. This allows you to perform actions specific to the game selection, such as loading a specific configuration or starting a game.

### Configuration Structure:

Each macro definition follows this precise format:
```
G[9-20]_DIRECTION[_MOD] = "shell command"
```

### Example Configuration File

```toml
[tf2]
G1_DOWN = "echo 'TF2 - G1 pressed'"
G2_UP = "xdotool key ctrl+c"

[overwatch]
G9_DOWN = "echo 'Overwatch - G9 pressed'"
ON_ACTIVATE = "game-start.sh overwatch"

[finals]
G10_DOWN_MOD = "echo 'Finals - G10 with modifier'"
G15_UP = "killall firefox"
```

## Requirements

This tool requires:
- Rust (stable version)
- Root access or udev rules to read `/dev/input` devices

To run without root privileges, you can set up udev rules to grant read access to the input device:
```bash
sudo cp 99-g600.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## Troubleshooting

- *Error: Permission denied*: Make sure you're either running as root or have proper udev permissions.
- *No input detected*: Confirm that the correct device path is being used with `ls /dev/input/by-id`.

## Testing

Run tests using:
```bash
cargo test
```

## Contributing

Feel free to open issues or submit pull requests on [GitHub](https://github.com/danhab99/logitech-g600-rs).

## License

MIT License

---

*Note: This project is intended for personal use and educational purposes. Always ensure you comply with the terms of service of your gaming platforms and hardware manufacturers.*
