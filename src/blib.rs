#![allow(unused)]
use gdnative::{prelude::*};
use std::{thread, time};
use futures::executor::block_on;

use std::sync::mpsc::{SyncSender, Receiver, sync_channel};


// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {

    handle.add_class::<Tunnel>();
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);


// Tunnel Stuff
#[derive(NativeClass)]
#[inherit(Node)]
pub struct Tunnel {
    sender: SyncSender<i32>,
    receiver: Receiver<i32>,
}
// Add crossbeam channels

impl Tunnel {
    fn new(_base: &Node) -> Self {

        let (sender, receiver) = sync_channel::<i32>(10);
        

        Tunnel {
            sender,
            receiver
        }
    }
}

#[methods]
impl Tunnel {

    #[method]
    fn _ready(&self, #[base] base: &Node) {
        godot_print!("Starting Thread!");

        let stx = self.sender.clone();

        // let (sender, reciever) = channel::<i32>();
        thread::spawn(move || {
            thread::sleep(time::Duration::from_secs(8));
            let res = stx.send(69);
            match res {
                Err(e) => {
                    godot_print!("{:?}", e);
                }
                Ok(_) => {
                    godot_print!("Successfully sent {}", 69);
                }
            }
        });
    }

    #[method]
    fn _process(&self, #[base] base: &Node, _delta: f64) {
        let res = self.receiver.recv_timeout(time::Duration::from_millis(500));

        match res {
            Ok(x) => {
                godot_print!("Received: {}", x);
            },
            Err(_) => {

            }
        }
    }

    // Process method should communicate with async thread through channels

}