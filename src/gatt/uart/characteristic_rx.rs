use bluster::gatt::{
    characteristic::{Characteristic, Properties, Secure, Write},
    descriptor::Descriptor,
    event::{Event, Response},
};
use futures::{prelude::*, sync::mpsc::channel};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::runtime::current_thread::Runtime;
use uuid::Uuid;

const RX_UUID: &str = "6e400002-b5a3-f393-e0a9-e50e24dcca9e";

pub fn create_rx(
    runtime: &Arc<Mutex<Runtime>>,
    descriptors: HashSet<Descriptor>,
) -> Characteristic {
    let (sender, receiver) = channel(1);
    runtime
        .lock()
        .unwrap()
        .spawn(receiver.for_each(move |event| {
            if let Event::WriteRequest(write_request) = event {
                println!("RX <- {:?}", write_request.data);
                if !write_request.without_response {
                    write_request
                        .response
                        .send(Response::Success(vec![]))
                        .unwrap();
                }
            }
            Ok(())
        }));

    Characteristic::new(
        Uuid::parse_str(RX_UUID).unwrap(),
        Properties::new(
            None,                                                      /* read */
            Some(Write::WithResponse(Secure::Secure(sender.clone()))), /* write */
            None,                                                      /* notify */
            None,                                                      /* indicate */
        ),
        None, /* initial value */
        descriptors,
    )
}
