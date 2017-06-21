extern crate pcap;
extern crate time;
extern crate sonos_api;

use pcap::Device;
use std::thread;
use std::sync::mpsc;
use std::fmt::LowerHex;
use std::fmt::Formatter;
use std::collections::HashMap;
use std::fmt;
use sonos_api::SonosApi;
use sonos_api::SonosRoom;

struct HexSlice<'a>(&'a [u8]);

impl<'a> HexSlice<'a> {
    fn new<T>(data: &'a T) -> HexSlice<'a>
        where T: ?Sized + AsRef<[u8]> + 'a
    {
        HexSlice(data.as_ref())
    }
}

// You can even choose to implement multiple traits, like Lower and UpperHex
impl<'a> fmt::Display for HexSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, byte) in self.0.iter().enumerate() {
            // Decide if you want to pad out the value here
            write!(f, "{:02x} ", byte)?;
            if (i + 1) % 8 == 0 {
                write!(f, "\n");
            }else if (i + 1) % 4 == 0 {
                write!(f, "  ");
            }
        }
        Ok(())
    }
}
fn main() {
    let (tx, rx) = mpsc::channel();

    let mut mac_to_name = HashMap::new();
    let mut last_call : HashMap<String, i64> = HashMap::new();
    let mut room : SonosRoom = SonosApi::new("http://localhost:5005".to_string()).room("Arbeiterklasse".to_string());

    mac_to_name.insert([0x0, 0x28, 0xf8, 0x4c, 0x7b ,0x2e], String::from("Oskar's Thinkpad"));

    thread::spawn(move || {
        let mut cap = Device::lookup().unwrap().open().unwrap();

        while let Ok(packet) = cap.next() {
            let len = (&packet.data).len();
            let mut data = Vec::new();
            data.resize(len, 0);
            data.clone_from_slice(&packet);
            tx.send(data).unwrap();
        }
    });

    while let Ok(packet) = rx.recv() {
        let dest_hwadr = &packet[0..6];
        let source_hwadr = &packet[6..12];

        match mac_to_name.get(source_hwadr) {
            Some(name) => {
                call(&mut last_call, name, &mut room);
            }
            None => {}
        }

        match mac_to_name.get(dest_hwadr) {
            Some(name) => {
                call(&mut last_call, name, &mut room);
            }
            None => {}
        }
    }
}

fn call(last_call : &mut HashMap<String, i64>, caller: &String, room: &mut SonosRoom) {
    let mut update = false;

    match last_call.get(caller) {
        Some(last_send) => {
            if last_send + 10 < time::now().to_timespec().sec {
                let greeting = format!{"Hello {}", caller};
                room.say(greeting);
                update = true;
            }
        }
        None => {
            let greeting = format!{"Hello {}", caller};
            room.say(greeting);
            update = true;
        }
    }

    if update {
        last_call.insert(caller.clone(), time::now().to_timespec().sec);
    }
}