// jkcoxson

use std::{fs::File, io::Write};

use directories::UserDirs;
use rusty_libimobiledevice::{libimobiledevice::get_devices, userpref::read_pair_record};

fn main() {
    println!("Fetching connected devices...");
    let devices = match get_devices() {
        Ok(devices) => devices,
        Err(e) => {
            println!(
                "Unable to get devices, have you installed iTunes and started it? - {:?}",
                e
            );
            pause();
            return;
        }
    };
    if devices.len() == 0 {
        println!("No devices found, plug in your device or check the cable.");
        pause();
        return;
    }
    let mut device = None;
    for i in devices {
        if i.get_network() == false {
            device = Some(i);
            break;
        }
    }
    if device.is_none() {
        println!("No devices found, plug in your device or check the cable.");
        pause();
        return;
    }
    let device = device.unwrap();

    let lockdown = match device.new_lockdownd_client("jit_extractor".to_string()) {
        Ok(l) => l,
        Err(e) => {
            println!("Unable to start lockdown client {:?}", e);
            pause();
            return;
        }
    };

    loop {
        println!("Fetching pair record...");
        match read_pair_record(device.get_udid().clone()) {
            Ok(mut pair_record) => {
                // Remove escrow keybag from pair record
                pair_record.dict_remove_item("EscrowBag").unwrap();
                // Add udid to pair record
                pair_record
                    .dict_set_item("UDID", device.get_udid().into())
                    .unwrap();
                // Save pair_record to file
                let user_dirs = UserDirs::new().unwrap();
                let desktop_dir = user_dirs.desktop_dir().unwrap();
                let mut file = match File::create(
                    desktop_dir.join(format!("{}.mobilepairingfile", device.get_udid())),
                ) {
                    Ok(f) => f,
                    Err(e) => {
                        println!("Unable to create file {:?}", e);
                        pause();
                        return;
                    }
                };
                match file.write_all(pair_record.to_string().as_bytes()) {
                    Ok(_) => println!(
                        "Pair record saved to your desktop at {}.mobilepairingfile",
                        device.get_udid()
                    ),
                    Err(e) => {
                        println!("Unable to write to file {:?}", e);
                        pause();
                        return;
                    }
                }
                pause();
                break;
            }
            Err(e) => {
                println!("Unable to read pair record {:?}", e);
                println!("Pairing...");
                match lockdown.pair(None) {
                    Ok(_) => println!("Paired!"),
                    Err(e) => {
                        println!("Unable to pair {:?}", e);
                        pause();
                        return;
                    }
                };
                pause();
            }
        }
    }
}

fn pause() {
    println!("Press enter to continue...");
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).unwrap();
}
