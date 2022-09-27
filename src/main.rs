//! An example showing off the usage of `Deserialize` to automatically decode
//! TOML into a Rust `struct`

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::process::Command;
extern crate clap;
use clap::{App, Arg};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    pout: HashMap<String, SetConf>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SetConf {
    path: String,
    val: String,
}

const G_CONF_PATH: &str = "/root/lin_config.toml";

fn get_cur_conf(path: String) -> String {
    let mut file = match File::open(path) {
        Ok(file_fd) => file_fd,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                return "".to_string();
            }
            _ => {
                panic!("other err: {:?}", e);
            }
        },
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read the file");
    return contents;
}

fn save_string2file(path: String, val: String) {
    let proc_id = process::id().to_string();
    let bak_file_path = path.clone() + &proc_id;
    let _ = fs::remove_file(bak_file_path.clone());

    {
        let mut fd = File::create(bak_file_path.clone()).unwrap();
        let _ = fd.write_all(val.as_bytes());
        fd.sync_all().unwrap();
    }

    let _ = Command::new("mv")
        .arg(bak_file_path)
        .arg(path)
        .output()
        .expect("failed to mv");
}

fn effect_one_conf(conf_path:String, conf_val:String){
    if let Ok(mut file_fd) = File::create(conf_path.clone()){
        //println!("will write {} {}", conf_path, conf_val);
        let e = file_fd.write(conf_val.as_bytes());
        //println!("write ret :{:?}", e);
    }
}


fn effect_conf(conf : Config){
    for (key_str, v) in conf.pout{
        let conf_path = v.path;
        let conf_val = v.val;
        effect_one_conf(conf_path, conf_val);
    }
}

fn main() {
    let matches = App::new("init conf set")
        .version("0.1.0")
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .takes_value(true)
                .help("key of config"),
        )
        .arg(
            Arg::with_name("value")
                .short("v")
                .long("value")
                .takes_value(true)
                .help("value of config"),
        )
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .takes_value(true)
                .help("path of config"),
        )
        .arg(
            Arg::with_name("effect")
                .short("e")
                .long("effect")
                .takes_value(false)
                .help("take effect"),
        )
        .get_matches();

    let contents = get_cur_conf(G_CONF_PATH.to_string());
    let mut decoded = match toml::from_str::<Config>(&contents) {
        Ok(decoded) => decoded,
        Err(e) => {
            panic!("config file err...{:?}", e);
        }
    };

    let mut conf_path = "".to_string();

    match matches.value_of("path") {
        Some(path_str) => {
            conf_path = path_str.to_string();
        },
        _ =>{},
    }

    match matches.value_of("key") {
        None => {
            if matches.is_present("effect") {
                effect_conf(decoded);
            }else{
                panic!("key is not set");
            }
            return;
        }
        Some(key_str) => {
            if let Some(val_str) = matches.value_of("value") {
                //println!("get key {} val: {}", key_str, val_str);

                decoded.pout.get_mut(&key_str.to_string()).unwrap().val = val_str.to_string();

                if conf_path.len() != 0 {
                    decoded.pout.get_mut(&key_str.to_string()).unwrap().path = conf_path;
                }

                let save_str = toml::to_string(&decoded).unwrap();
                save_string2file(G_CONF_PATH.to_string(), save_str);

                effect_one_conf(decoded.pout.get_mut(&key_str.to_string()).unwrap().path.clone(), val_str.to_string());
            }else{
                panic!("val is not set");
            }
        }
    }
}
