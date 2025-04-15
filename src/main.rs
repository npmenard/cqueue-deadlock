use std::{future::Future, task::Poll, time::Duration};

use async_io::block_on;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use futures_lite::StreamExt;

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

struct FutureExpample {
    large: [u32; 200],
}

impl Future for FutureExpample {
    type Output = u32;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let will_complete = unsafe { esp_idf_svc::sys::esp_random() } % 2;
        if will_complete == 1 {
            return Poll::Ready(
                self.large[unsafe { esp_idf_svc::sys::esp_random() % (self.large.len() as u32 - 1) }
                    as usize],
            );
        }
        Poll::Pending
    }
}

async fn runner() {
    use async_io::Timer;
    use futures_util::stream::FuturesUnordered;

    loop {
        let mut futures = FuturesUnordered::new();
        for _ in 0..100 {
            futures.push(async {
                let fut = FutureExpample {
                    large: [0xDEADBEFF_u32; 200],
                };
                futures_lite::future::or(fut, async {
                    let _ = Timer::after(Duration::from_millis(unsafe {
                        esp_idf_svc::sys::esp_random() % 1000
                    } as u64))
                    .await;
                    0
                })
                .await
            });
        }
        while let Some(_) = futures.next().await {}
        log::info!("next loop");
    }
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::sys::esp!(unsafe {
        esp_idf_svc::sys::esp_vfs_eventfd_register(&esp_idf_svc::sys::esp_vfs_eventfd_config_t {
            max_fds: 5,
        })
    })
    .unwrap();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
        sys_loop,
    )
    .unwrap();

    start_wifi(&mut wifi);

    let ip_info = wifi.wifi().sta_netif().get_ip_info().unwrap();

    log::info!("Wifi DHCP info: {:?}", ip_info);
    let exec = async_executor::LocalExecutor::new();

    block_on(exec.run(runner()));

    log::info!("Hello, world!");
}

fn start_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        bssid: None,
        auth_method: esp_idf_svc::wifi::AuthMethod::WPA2Personal,
        password: PASSWORD.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration).unwrap();

    wifi.start().unwrap();
    log::info!("Wifi started");

    wifi.connect().unwrap();
    log::info!("Wifi connected");

    wifi.wait_netif_up().unwrap();
    log::info!("Wifi netif up");
}
