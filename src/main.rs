use std::{error::Error, fs, path::PathBuf, process::Command, str::FromStr};

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

const CONFIG_FILE_NAME: &str = "ev-cmd.toml";

fn main() -> Result<(), Box<dyn Error>> {
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

    let config = match maybe_config {
        Some(c) => c,
        None => {
            panic!("didn't find config file");
        }
    };

    let mut device = Device::open(args.device_path)?;

    loop {
        let events = device.fetch_events()?;

        for event in events {
            let code = event.code();
            let state = event.value();

            if state != 1 || code == 0 {
                continue;
            }

            let key = format!("{}", code);
            println!("Caught keycode {}", key);

            let cmd = config.get(key.as_str());
            match cmd {
                None => {}
                Some(cmd) => match cmd.as_str() {
                    Some(c_raw) => {
                        let c = Command::new("/bin/sh").args(["-c", c_raw]).output();
                        match c {
                            Ok(o) => {
                                println!(
                                    "Executed \"{}\": {:?}",
                                    c_raw,
                                    String::from_utf8(o.stdout)
                                );
                            }
                            Err(e) => {
                                println!("Failed to execute \"{}\": {}", c_raw, e);
                            }
                        }
                    }
                    None => {}
                },
            }
        }
    }
}
