extern crate getopts;
extern crate librespot;
extern crate rpassword;

use getopts::Options;
use rpassword::read_password;
use std::clone::Clone;
use std::fs::File;
use std::io::{stdout, Read, Write};
use std::path::{Path, PathBuf};
use std::thread;

use librespot::discovery::DiscoveryManager;
use librespot::player::Player;
use librespot::session::{Config, Session};
use librespot::spirc::SpircManager;
use librespot::util::version::version_string;

fn usage(program: &str, opts: &Options) -> String {
    let brief = format!("Usage: {} [options]", program);
    format!("{}", opts.usage(&brief))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.reqopt("a", "appkey", "Path to a spotify appkey", "APPKEY");
    opts.reqopt("u", "username", "Username to sign in with", "USERNAME");
    opts.optopt("p", "password", "Password (optional)", "PASSWORD");
    opts.reqopt("c", "cache", "Path to a directory where files will be cached.", "CACHE");
    opts.reqopt("n", "name", "Device name", "NAME");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(f) => { 
                print!("Error: {}\n{}", f.to_string(), usage(&*program, &opts));
                return;
        }
    };

    let appkey = {
        let mut file = File::open(
            Path::new(&*matches.opt_str("a").unwrap())
        ).expect("Could not open app key.");

        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        data
    };

    let username = matches.opt_str("u").unwrap();
    let cache_location = matches.opt_str("c").unwrap();
    let name = matches.opt_str("n").unwrap();

    let password = matches.opt_str("p").unwrap_or_else(|| {
        print!("Password: "); 
        stdout().flush().unwrap();
        read_password().unwrap()
    });

    let config = Config {
        application_key: appkey,
        user_agent: version_string(),
        device_name: name,
        cache_location: PathBuf::from(cache_location)
    };

    let session = Session::new(config);
    //session.login_password(username, password).unwrap();
    let mut discovery = DiscoveryManager::new(session.clone());
    discovery.run();

    let player = Player::new(session.clone());
    let mut spirc = SpircManager::new(session.clone(), player);
    thread::spawn(move || {
        spirc.run()
    });

    loop {
        session.poll();
    }
}

