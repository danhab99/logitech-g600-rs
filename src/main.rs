use std::{
    collections::HashMap, error::Error, fs, path::PathBuf, process::Command, str::FromStr, thread,
};

use clap::Parser;
use evdev::Device;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    device_path: String,

    #[arg(short, long)]
    config_path: Option<String>,
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
        (0u16, "9"),
        (0u16, "10"),
        (0u16, "11"),
        (0u16, "12"),
        (0u16, "13"),
        (0u16, "14"),
        (0u16, "15"),
        (0u16, "16"),
        (0u16, "17"),
        (0u16, "18"),
        (0u16, "19"),
        (0u16, "20"),
        (0u16, "UP"),
        (0u16, "DOWN"),
        (0u16, "MOD"),
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

    let config_names = config.keys().collect::<Vec<_>>();
    let mut current_config_index = 0i16;
    let mut last_config_index = 0i16;

    let mut mod_activated = false;
    loop {
        let events = device.fetch_events()?;

        for event in events {
            let code = keycode_name[&event.code()];
            let state = match event.value() {
                0 => continue,
                1 => "DOWN",
                2 => "UP",
                _ => panic!("unknown event"),
            };

            let current_config = some_or!(
                config.get(config_names[current_config_index as usize]),
                panic!("No configs found")
            );

            if current_config_index != last_config_index {
                match current_config.get("OnActivate") {
                    None => (),
                    Some(activate) => match activate.as_str() {
                        None => (),
                        Some(cmd) => execute(cmd),
                    },
                }
                last_config_index = current_config_index;
            }

            match code {
                "MOD" => {
                    mod_activated = state == "DOWN";
                    continue;
                }
                "UP" => {
                    current_config_index += 1;
                    if current_config_index > config_names.len() as i16 {
                        current_config_index = 0;
                    }
                    continue;
                }
                "DOWN" => {
                    current_config_index -= 1;
                    if current_config_index < 0 {
                        current_config_index = config_names.len() as i16;
                    }
                    continue;
                }
                _ => (),
            }

            let mut key = vec![code, state];
            if mod_activated {
                key.push("MOD");
            }

            let k = key.join("_");
            println!("Caught macro {}", k);

            let cmd = some_or!(current_config.get(k.as_str()), continue);
            let c = some_or!(cmd.as_str(), continue);
            execute(c);
        }
    }
}
