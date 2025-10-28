// serial.rs

use embedded_svc::io::asynch::Write;
use esp_idf_hal::{
    gpio::{AnyIOPin, PinDriver},
    uart,
    units::Hertz,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc},
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

    // create a broadcast channel for sending serial msgs to all clients
    let (ser_read_tx, _) = broadcast::channel(CHANSZ);

    // create an mpsc channel for receiving serial port input from any client
    // mpsc = multi-producer, single consumer queue
    let (ser_write_tx, ser_write_rx) = match state.config.serial_write_enabled {
        true => {
            let c = mpsc::channel(CHANSZ);
            (Some(c.0), Some(c.1))
        }
        false => (None, None),
    };

    let _ = tokio::try_join!(
        Box::pin(handle_network(
            state.clone(),
            ser_read_tx.clone(),
            ser_write_tx,
        )),
        Box::pin(handle_serial(state, ser_read_tx, ser_write_rx))
    );
    // if any of the above tasks fail, we return and main() will reboot the whole system
    Ok(())
}

async fn handle_serial(
    state: Arc<Pin<Box<MyState>>>,
    ser_read_tx: broadcast::Sender<Vec<u8>>,
    ser_write_rx: Option<mpsc::Receiver<Vec<u8>>>,
) -> anyhow::Result<()> {
    info!("UART1 initialization...");

    let bps = state.config.bps;
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

    // create a dummy rx pair if we did not get one
    let mut write_rx = ser_write_rx.unwrap_or_else(|| mpsc::channel(1).1);

    let mut buf = [0; BUFSZ];
    loop {
        tokio::select! {
            Some(msg) = write_rx.recv() => {
                led.toggle().ok();
                // info!("serial write {} bytes", msg.len());
                uart.write_all(msg.as_ref()).await?;

            }

            res = uart.read(&mut buf) => {
                match res {
                    Ok(0) => {
                        info!("Serial <EOF>");
                        break;
                    }
                    Ok(n) => {
                        led.toggle().ok();
                        // info!("Serial read {n} bytes.");
                        ser_read_tx.send(buf[0..n].to_owned())?;
                    }
                    Err(e) => {
                        bail!(e);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_network(
    state: Arc<Pin<Box<MyState>>>,
    ser_read_tx: broadcast::Sender<Vec<u8>>,
    ser_write_tx: Option<mpsc::Sender<Vec<u8>>>,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", state.config.serial_tcp_port)).await?;
    info!("Serial server listening...");

    let write_enabled = ser_write_tx.is_some();
    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((stream, addr)) => {
                let cnt = state.api_cnt.fetch_add(1, Ordering::Relaxed);

                info!("Client #{cnt} connected from {}:{}", addr, addr.port());
                let ser_read_rx = ser_read_tx.subscribe();
                let ser_write_tx_c = ser_write_tx.clone();
                tokio::spawn(async move {
                    Box::pin(handle_client(
                        cnt,
                        stream,
                        ser_read_rx,
                        ser_write_tx_c,
                        write_enabled,
                    ))
                    .await
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
    c: u32,
    mut sock: TcpStream,
    mut ser_read_rx: broadcast::Receiver<Vec<u8>>,
    ser_write_tx: Option<mpsc::Sender<Vec<u8>>>,
    write_enabled: bool,
) -> anyhow::Result<()> {
    let mut buf = [0; BUFSZ];

    loop {
        tokio::select! {
            Ok(msg) = ser_read_rx.recv() => {
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
                // the data read from tcp sucket is thrown away unless serial write is enabled
                if write_enabled {
                    ser_write_tx.as_ref().unwrap().send(buf[0..n].to_owned()).await?;
                }
            }
        }
    }
}
// EOF
