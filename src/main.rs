use std::{
    collections::HashMap, error::Error, fs, path::PathBuf, process::Command, str::FromStr, thread,
};

use clap::Parser;
use evdev::Device;
use toml::value;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    device_path: String,

    #[arg(short, long)]
    config_path: Option<String>,

    #[arg(short, long)]
    xdebug: bool,
}

const CONFIG_FILE_NAME: &str = "g600.toml";

macro_rules! some_or {
    // Case where the expression is `Some`, it unpacks and returns the value.
    ($expr:expr, break) => {
        match $expr {
            Some(val) => val,
            None => break,
        }
    };

    // Case where the expression is `Some`, it unpacks and returns the value.
    ($expr:expr, continue) => {
        match $expr {
            Some(val) => val,
            None => continue,
        }
    };

    ($expr:expr, $none:expr) => {
        match $expr {
            Some(val) => val,
            None => $none,
        }
    };
}

fn execute(c: &str) {
    let cmd = String::from(c);
    thread::spawn(move || {
        let c = Command::new("/bin/sh").args(["-c", cmd.as_str()]).output();
        match c {
            Ok(o) => {
                println!("Executed \"{}\": {:?}", cmd, String::from_utf8(o.stdout));
            }
            Err(e) => {
                println!("Failed to execute \"{}\": {}", cmd, e);
            }
        }
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    let keycode_name = HashMap::from([
        (30u16, "G9"),
        (48u16, "G10"),
        (46u16, "G11"),
        (32u16, "G12"),
        (18u16, "G13"),
        (33u16, "G14"),
        (34u16, "G15"),
        (35u16, "G16"),
        (23u16, "G17"),
        (36u16, "G18"),
        (37u16, "G19"),
        (38u16, "G20"),
        (50u16, "UP"),
        (49u16, "DOWN"),
        (25u16, "MOD"),
    ]);

    let args = Args::parse();

    let mut cwd_config_path = std::env::current_dir().unwrap();
    cwd_config_path.push(CONFIG_FILE_NAME);

    let mut config_config_path = dirs::config_dir().unwrap();
    config_config_path.push(CONFIG_FILE_NAME);

    let mut etc_config_path = PathBuf::from_str(CONFIG_FILE_NAME)?;
    etc_config_path.push(CONFIG_FILE_NAME);

    let config_paths = [
        PathBuf::from_str(&args.config_path.unwrap_or(String::new()))?,
        cwd_config_path,
        config_config_path,
        etc_config_path,
    ];

    let mut maybe_config: Option<toml::Table> = None;

    for config_path in config_paths {
        if config_path.exists() {
            let raw = fs::read(config_path)?;
            let content = String::from_utf8(raw)?;
            maybe_config = Some(content.parse::<toml::Table>()?);
            break;
        }
    }

    let config = some_or!(maybe_config, panic!("didn't find config file"));

    let mut device = Device::open(args.device_path)?;

    if args.xdebug {
        loop {
            let events = device.fetch_events()?;
            for event in events {
                let code = &event.code();
                let state = event.value();
                println!("code={} state={}", code, state);
            }
        }
    }

    let config_names = config.keys().collect::<Vec<_>>();
    let mut current_config_index = 0i16;
    let mut mod_activated = false;
    let mut changed = false;
    // let mut last_code = 0u16;
    // let mut last_value = 0i32;

    loop {
        let events = device.fetch_events()?;

        for event in events {
            let c = event.code();
            let v = event.value();

            let state = match v {
                1 => "DOWN",
                0 => "UP",
                _ => continue,
            };

            if state == "DOWN" && c == 0 {
                continue;
            }
            if c == 0 && v == 0 {
                continue;
            }

            println!("Keypress code={} value={}", c, v);
            let code = keycode_name[&c];

            let current_config = some_or!(
                config.get(config_names[current_config_index as usize]),
                panic!("No configs found")
            );

            match code {
                "MOD" => {
                    mod_activated = state == "DOWN";
                    println!("Mod changed {}", mod_activated);
                }
                "UP" => {
                    current_config_index += 1;
                    if current_config_index >= config_names.len() as i16 {
                        current_config_index = 0;
                    }
                    changed = true;
                    println!("Change profile up {}", current_config_index);
                }
                "DOWN" => {
                    current_config_index -= 1;
                    if current_config_index < 0 {
                        current_config_index = (config_names.len() as i16) - 1;
                    }
                    changed = true;
                    println!("Change profile down {}", current_config_index);
                }
                _ => {
                    let mut key = vec![code, state];
                    if mod_activated {
                        key.push("MOD");
                    }

                    let k = key.join("_");
                    println!("Caught keypress {}", k);

                    let cmd = some_or!(current_config.get(k.as_str()), continue);
                    let c = some_or!(cmd.as_str(), continue);
                    execute(c);
                }
            }

            if changed {
                let current_config = some_or!(
                    config.get(config_names[current_config_index as usize]),
                    panic!("No configs found")
                );

                match current_config.get("OnActivate") {
                    None => (),
                    Some(activate) => match activate.as_str() {
                        None => (),
                        Some(cmd) => {
                            println!("Executing activation command {}", cmd);
                            execute(cmd)
                        }
                    },
                }
            }
        }
    }
}
