use std::{
    collections::HashMap,
    error::Error,
    fs,
    fs::File,
    path::PathBuf,
    process::{self, Command},
    str::FromStr,
    thread,
};

use clap::Parser;
use evdev::Device;
use fs2::FileExt;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    device_path: String,

    #[arg(short, long)]
    config_path: Option<String>,

    #[arg(short = 'x', long, default_value_t = false)]
    debug: bool,

    #[arg(short, long)]
    lock_file: Option<String>,
}

const CONFIG_FILE_NAME: &str = "g600.toml";

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

    let x = args.lock_file.unwrap_or("/tmp/logitech-g600-rs.pid.lock".to_string());
    let lock_path = PathBuf::from_str(x.as_str())?;
    let file = File::create(&lock_path).expect("Could not create lock file");
    if let Err(e) = file.try_lock_exclusive() {
        println!("Another instance is already running: {e}");
        process::exit(1);
    }

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

    let config = maybe_config.ok_or("didn't find config file")?;

    let mut device = Device::open(args.device_path)?;

    if args.debug {
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

            if state == "DOWN" && c == 0 && v == 0 {
                continue;
            }

            println!("Keypress code={} value={}", c, v);
            let code = match keycode_name.get(&c) {
                None => continue,
                Some(x) => *x,
            };

            let current_config = config
                .get(config_names[current_config_index as usize])
                .ok_or("unable to find config")?;

            match code {
                "MOD" => {
                    mod_activated = state == "DOWN";
                    println!("Mod changed {}", mod_activated);
                }
                "UP" => {
                    if state == "DOWN" {
                        current_config_index += 1;
                        if current_config_index >= config_names.len() as i16 {
                            current_config_index = 0;
                        }
                        changed = true;
                        println!("Change profile up {}", current_config_index);
                    }
                }
                "DOWN" => {
                    if state == "DOWN" {
                        current_config_index -= 1;
                        if current_config_index < 0 {
                            current_config_index = (config_names.len() as i16) - 1;
                        }
                        changed = true;
                        println!("Change profile down {}", current_config_index);
                    }
                }
                _ => {
                    let mut key = vec![code, state];
                    if mod_activated {
                        key.push("MOD");
                    }

                    let k = key.join("_");
                    println!("Caught keypress {}", k);

                    match current_config.get(k.as_str()).and_then(|cmd| cmd.as_str()) {
                        Some(c) => execute(c),
                        None => continue,
                    }
                }
            }

            if changed {
                changed = false;

                match config
                    .get(config_names[current_config_index as usize])
                    .and_then(|current_config| current_config.get("ON_ACTIVATE"))
                    .and_then(|activate| activate.as_str())
                {
                    Some(cmd) => {
                        println!("Executing activation command {}", cmd);
                        execute(cmd);
                    }
                    None => {}
                }
            }
        }
    }
}
