// serial.rs

use anyhow::bail;
use embedded_svc::io::asynch::Write;
use esp_idf_hal::{
    gpio::{AnyIOPin, PinDriver},
    uart,
    units::Hertz,
};
use log::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc},
    time::{sleep, Duration},
};

use crate::*;

const BUFSZ: usize = 64;
const CHANSZ: usize = 8;

pub async fn run_serial(state: Arc<Pin<Box<MyState>>>) -> anyhow::Result<()> {
    info!("Waiting for WiFi...");
    loop {
        if *state.wifi_up.read().await {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }

    // Note: here read/write in variable naming is referring to the serial port data direction

    // create a broadcast channel for sending serial msgs to all clients
    let (read_tx, _read_rx) = broadcast::channel(CHANSZ);

    // create an mpsc channel for receiving serial port input from any client
    // mpsc = multi-producer, single consumer queue
    let (write_tx, write_rx) = mpsc::channel(CHANSZ);

    let _ = tokio::try_join!(
        Box::pin(handle_network(state.clone(), read_tx.clone(), write_tx,)),
        Box::pin(handle_serial(state, read_tx, write_rx))
    );
    // if any of the above tasks fail, we return and main() will reboot the whole system
    Ok(())
}

async fn handle_serial(
    state: Arc<Pin<Box<MyState>>>,
    a_send: broadcast::Sender<Vec<u8>>,
    mut a_recv: mpsc::Receiver<Vec<u8>>,
) -> anyhow::Result<()> {
    info!("UART1 initialization...");

    let bps = state.config.read().await.bps;
    use esp_idf_hal::uart::config::*;
    let ser_config = Config::new()
        .flow_control(FlowControl::None)
        .parity_none()
        .data_bits(DataBits::DataBits8)
        .stop_bits(StopBits::STOP1)
        .baudrate(Hertz(bps));
    info!("UART1 config:\n{ser_config:#?}");

    let my_ser = state.serial.write().await.take().unwrap();
    let mut uart = uart::AsyncUartDriver::new(
        my_ser.uart,
        my_ser.tx,
        my_ser.rx,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &ser_config,
    )?;
    let mut led = PinDriver::output(my_ser.led)?;

    info!("UART1 opened.");

    let mut buf = [0; BUFSZ];
    loop {
        tokio::select! {
            Some(msg) = a_recv.recv() => {
                led.toggle().ok();
                // info!("serial write {} bytes", msg.len());
                uart.write_all(msg.as_ref()).await?;

            }

            res = uart.read(&mut buf) => {
                match res {
                    Ok(0) => {
                        info!("Serial <EOF>");
                        return Ok(());
                    }
                    Ok(n) => {
                        led.toggle().ok();
                        // info!("Serial read {n} bytes.");
                        a_send.send(buf[0..n].to_owned())?;
                    }
                    Err(e) => {
                        bail!(e);
                    }
                }
            }
        }
    }
}

async fn handle_network(
    state: Arc<Pin<Box<MyState>>>,
    read_atx: broadcast::Sender<Vec<u8>>,
    write_atx: mpsc::Sender<Vec<u8>>,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:23").await?;
    info!("Serial server listening...");

    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((stream, addr)) => {
                let c = {
                    let mut c = state.cnt.write().await;
                    *c += 1;
                    *c
                };

                info!("Client #{c} connected from {}:{}", addr, addr.port());
                let client_read_atx = read_atx.subscribe();
                let client_write_atx = write_atx.clone();
                tokio::spawn(async move {
                    Box::pin(handle_client(c, stream, client_read_atx, client_write_atx)).await
                });
            }
            Err(e) => {
                error!("Accept failed: {e}");
            }
        }
    }

    // Ok(())
}

async fn handle_client(
    c: u64,
    mut sock: TcpStream,
    mut rx: broadcast::Receiver<Vec<u8>>,
    tx: mpsc::Sender<Vec<u8>>,
) -> anyhow::Result<()> {
    let mut buf = [0; BUFSZ];

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                sock.write_all(msg.as_ref()).await?;
                sock.flush().await?;
            }

            res = sock.read(&mut buf) => {
                let n = match res {
                    Err(e) => {
                        error!("Client #{c} error: {e}");
                        bail!(e);
                    },
                    Ok(x) => x
                };

                if n == 0 {
                    info!("Client #{c} disconnected");
                    return Ok(());
                }

                tx.send(buf[0..n].to_owned()).await?;
            }
        }
    }
}
// EOF
