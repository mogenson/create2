mod ble;

use self::ble::create_uart;
use bluster::Peripheral;
use futures::{future, prelude::*};
use futures_timer::FutureExt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::current_thread::Runtime;
use uuid::Uuid;

const ADVERTISING_NAME: &str = "Create 2";
const ADVERTISING_UUID: &str = "48c5d828-ac2a-442d-97a3-0c9822b04979";

fn main() {
    println!("starting up");

    let runtime = Arc::new(Mutex::new(Runtime::new().unwrap()));

    // Create peripheral
    let peripheral_future = Peripheral::new(&runtime);
    let peripheral = Arc::new({ runtime.lock().unwrap().block_on(peripheral_future).unwrap() });
    peripheral.add_service(&create_uart(&runtime)).unwrap();

    // Create advertisement
    let advertisement = future::loop_fn(Arc::clone(&peripheral), |peripheral| {
        peripheral.is_powered().and_then(move |is_powered| {
            if is_powered {
                println!("peripheral powered on");
                Ok(future::Loop::Break(peripheral))
            } else {
                println!("peripheral pwered off");
                Ok(future::Loop::Continue(peripheral))
            }
        })
    })
    .timeout(Duration::from_secs(3))
    .and_then(|peripheral| {
        println!("start advertising");

        let peripheral2 = Arc::clone(&peripheral);
        peripheral
            .start_advertising(
                ADVERTISING_NAME,
                &[Uuid::parse_str(ADVERTISING_UUID).unwrap()],
            )
            .and_then(move |advertising_stream| Ok((advertising_stream, peripheral2)))
    })
    .and_then(|(advertising_stream, peripheral)| {
        println!("check advertising");

        let handled_advertising_stream = advertising_stream.for_each(|_| Ok(()));

        let advertising_check = future::loop_fn(Arc::clone(&peripheral), move |peripheral| {
            peripheral.is_advertising().and_then(move |is_advertising| {
                if is_advertising {
                    println!("peripheral available as \"{}\"", ADVERTISING_NAME);
                    Ok(future::Loop::Break(peripheral))
                } else {
                    println!("not advertising");
                    Ok(future::Loop::Continue(peripheral))
                }
            })
        });

        advertising_check.fuse().join(handled_advertising_stream)
    })
    .then(|_| {
        println!("something bad happened");
        Ok(())
    });

    // Spawn never ending process
    let mut runtime = runtime.lock().unwrap();
    runtime.spawn(advertisement);
    runtime.run().unwrap();
}
