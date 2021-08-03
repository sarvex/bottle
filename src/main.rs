// If you are running it in MacOS BigSur and later, you have to give permission to Bluetooth to your terminal in Security and privacy.
// More details in Btleplug README.md: https://github.com/deviceplug/btleplug

use btleplug::api::{Central, CentralEvent, Manager as _};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let manager = Manager::new().await?;

    // get the first bluetooth adapter
    // connect to the adapter
    let central = get_central(&manager).await;

    // Each adapter can only have one event receiver. We fetch it via
    // event_receiver(), which will return an option. The first time the getter
    // is called, it will return Some(Receiver<CentralEvent>). After that, it
    // will only return None.
    let mut events = central.events().await?;

    // start scanning for devices
    central.start_scan().await?;

    // Print based on whatever the event receiver outputs. Note that the event
    // receiver blocks, so in a real program, this should be run in its own
    // thread (not task, as this library does not yet use async channels).
    while let Some(event) = events.next().await {
        match event {
            CentralEvent::ManufacturerDataAdvertisement {
                address: _,
                manufacturer_data,
            } => {
                if let Some(data) = manufacturer_data.get(&0xffff_u16) {
                    let mut bytes = [0u8; 4];

                    // Copy the bytes reference to an *owned* array of four
                    // bytes...
                    //
                    // NOTE: The length of `data` MUST be 4! Otherwise this
                    // will panic.
                    bytes.copy_from_slice(data);

                    // ...so we can consume the bytes, and turn it into an
                    // integer. This is kind of like safely "casting" from
                    // a `[u8; 4]` to a u32.
                    //
                    // NOTE: James is GUESSING your bytes are in "Big
                    // Endian" format (this is the 'be' part), but if not,
                    // you will need something else, like `from_le_bytes`.
                    let new_data: u32 = u32::from_be_bytes(bytes);

                    // Now we can format that u32 into the output:
                    println!("Distance: {} units", new_data);
                }
            }
            _ => {}
        }
    }
    Ok(())
}
