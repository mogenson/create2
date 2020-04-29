use bluster::gatt::{
    characteristic::{Characteristic, Properties},
    descriptor::Descriptor,
    event::Event,
};
use futures::{prelude::*, sync::mpsc::channel};
use std::{
    collections::HashSet,
    sync::{atomic, Arc, Mutex},
    thread,
    time::Duration,
};
use tokio::runtime::current_thread::Runtime;
use uuid::Uuid;

const TX_UUID: &str = "6e400003-b5a3-f393-e0a9-e50e24dcca9e";

pub fn create_tx(
    runtime: &Arc<Mutex<Runtime>>,
    descriptors: HashSet<Descriptor>,
) -> Characteristic {
    let (sender, receiver) = channel(1);
    let notifying = Arc::new(atomic::AtomicBool::new(false));
    runtime
        .lock()
        .unwrap()
        .spawn(receiver.for_each(move |event| {
            match event {
                Event::NotifySubscribe(notify_subscribe) => {
                    println!("subscribed to TX characteristic");
                    let notifying = Arc::clone(&notifying);
                    notifying.store(true, atomic::Ordering::Relaxed);
                    thread::spawn(move || {
                        let mut tx_packet = vec![0u8; 20];
                        loop {
                            if !(&notifying).load(atomic::Ordering::Relaxed) {
                                break;
                            }

                            /* dummy work to generate packets */
                            for item in &mut tx_packet {
                                *item += 1;
                            }

                            println!("TX -> {:?}", tx_packet);

                            notify_subscribe
                                .clone()
                                .notification
                                .try_send(tx_packet.clone())
                                .unwrap();

                            thread::sleep(Duration::from_secs(1));
                        }
                    });
                }
                Event::NotifyUnsubscribe => {
                    println!("unsubscribed to TX characteristic");
                    notifying.store(false, atomic::Ordering::Relaxed);
                }
                _ => { /* ignore read and write */ }
            }
            Ok(())
        }));

    Characteristic::new(
        Uuid::parse_str(TX_UUID).unwrap(),
        Properties::new(
            None,                 /* read */
            None,                 /* write */
            Some(sender.clone()), /* notify */
            None,                 /* indicate */
        ),
        None, /* initial value */
        descriptors,
    )
}
