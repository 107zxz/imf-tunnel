#![allow(unused)]
use gdnative::{prelude::*};
use std::{thread, time};
use localtunnel::{open_tunnel, broadcast};

use std::sync::mpsc::{Sender, SyncSender, Receiver, sync_channel, channel};


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
    sender: SyncSender<String>,
    receiver: Receiver<String>,

    tunnel_notify_shutdown: Option<localtunnel::broadcast::Sender<()>>
}
// Add crossbeam channels

impl Tunnel {
    fn new(_base: &Node) -> Self {

        let (sender, receiver) = sync_channel::<String>(10);
        

        Tunnel {
            sender,
            receiver,
            tunnel_notify_shutdown: None
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
        thread::spawn(move || futures::executor::block_on(async {
            // thread::sleep(time::Duration::from_secs(8));

            // async_std::task::sleep(time::Duration::from_secs(8)).await;

            // Do thing
            // let (notify_shutdown, _) = broadcast::channel(1);

            let (ns, _) = broadcast::channel::<()>(1);

            // self.tunnel_notify_shutdown = Some(ns);

            let result = open_tunnel(
                Some("https://localtunnel.me"),
                Some("107zxz"),
                Some("localhost"),
                3000,
                ns.clone(),
                10,
            )
            .await
            .unwrap();

            // Print tunnel url
            stx.send(result);

            // Shut down after a minute
            async_std::task::sleep(time::Duration::from_secs(8)).await;


            // Shutdown the background tasks by sending a signal.
            let _ = ns.send(());


            // let res = stx.send(69);
            // match res {
            //     Err(e) => {
            //         godot_print!("{:?}", e);
            //     }
            //     Ok(_) => {
            //         godot_print!("Successfully sent {}", 69);
            //     }
            // }
        }));
    }

    #[method]
    fn _process(&self, #[base] base: &Node, _delta: f64) {
        let res = self.receiver.try_recv();

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