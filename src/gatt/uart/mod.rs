mod characteristic_rx;
mod characteristic_tx;
use self::characteristic_rx::create_rx;
use self::characteristic_tx::create_tx;
use bluster::gatt::service::Service;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::runtime::current_thread::Runtime;
use uuid::Uuid;

const UART_UUID: &str = "6e400001-b5a3-f393-e0a9-e50e24dcca9e";

pub fn create_uart(runtime: &Arc<Mutex<Runtime>>) -> Service {
    Service::new(
        Uuid::parse_str(UART_UUID).unwrap(),
        true, /* primary */
        {
            let mut characteristics = HashSet::new();
            characteristics.insert(create_tx(runtime, HashSet::new()));
            characteristics.insert(create_rx(runtime, HashSet::new()));
            characteristics
        },
    )
}
