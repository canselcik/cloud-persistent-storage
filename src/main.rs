#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rusoto;
extern crate aws_instance_metadata;
extern crate chrono;
extern crate serde_yaml;
#[macro_use]
extern crate serde_derive;
extern crate getopts;

use getopts::Options;
use std::env;

mod mkfs;
mod ebs;
mod mount;
mod config;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c",
                "config",
                "configuration file path (required)",
                "config.yml");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let config_path = match matches.opt_str("c") {
        Some(c) => c,
        None => {
            error!("configuration file path (-c, --config) must be provided");
            std::process::exit(100);

        }
    };
    info!("config path: {}", config_path);

    let config = match config::read_config_from_file(config_path.as_str()) {
        Ok(c) => c,
        Err(e) => {
            error!("failed to read configuration: {:?}", e);
            std::process::exit(100);
        }
    };
    info!("configuration: {:?}", config);

    // FIXME: config: mount point
    // FIXME: config: filesystem type
    // FIXME: windows platform support for mkfs/mount
    // FIXME: allocate volume if find_and_attach fails due to NoVolumesAvailable/AllAttachesFailed

    let block_device = "/dev/xvdh";

    match ebs::find_and_attach_volume(block_device) {
        Ok(_) => info!("attach volume succeeded"),
        Err(e) => {
            error!("attach volume failed: {:?}", e);
            std::process::exit(101);
        }
    }
    // FIXME: poll describe-volumes until .Volumes[0].Attachments[0].State == "attached"


    // FIXME: detect whether device has a filesystem on it
    match mkfs::make_filesystem(block_device) {
        Ok(_) => info!("created filesystem successfully"),
        Err(e) => {
            error!("failed to create filesystem: {:?}", e);
            std::process::exit(102);
        }
    }

    let mount_point = "/mnt/test";
    match std::fs::create_dir_all(mount_point) {
        Ok(_) => info!("created/ensured mount point directory successfully"),
        Err(e) => {
            error!("failed to create mount point directory: {:?}", e);
            std::process::exit(103);
        }
    }

    match mount::mount(block_device, mount_point) {
        Ok(_) => info!("mounted filesystem successfully"),
        Err(e) => {
            error!("failed to mount filesystem: {:?}", e);
            std::process::exit(104);
        }
    }
}

#[cfg(test)]
mod tests {}
