use clap::{Parser, Subcommand};
use hid_bridge::MessageType;
use hidapi::HidApi;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, time::Duration};

mod date_time;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    vid: u16,
    pid: u16,
    usage_page: u16,
    lat: f64,
    lon: f64,
    timezone: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Time,
    Wake,
    Heap,
    DebugOn,
    DebugOff,
    QueryFrameTime,
    FrameTime {
        time: u32,
    },
    QueryRgbHsv,
    SetRgbHsv {
        h: u16,
        s: u8,
        v: u8,
        #[arg(
            long,
            value_name = "BOOL",
            require_equals = true,
            num_args = 0..=1,
            default_missing_value = "false",
            value_enum
        )]
        save: bool,
    },
    DisplayBrightness {
        level: u8,
        #[arg(
            long,
            value_name = "BOOL",
            require_equals = true,
            num_args = 0..=1,
            default_missing_value = "false",
            value_enum
        )]
        save: bool,
    },
}

fn print_packet(label: &str, data: &[u8]) {
    print!("{label} ({} bytes): ", data.len());
    for b in data {
        print!("{b:02x} ");
    }
    print!(" |");
    for &b in data {
        let ch = if b.is_ascii_graphic() || b == b' ' {
            b as char
        } else {
            '.'
        };
        print!("{ch}");
    }
    println!("|");
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();

    let mut content = String::new();
    File::open("config.json5")?.read_to_string(&mut content)?;
    let config: Config = json5::from_str(&content)?;

    let api = HidApi::new()?;

    let info = api
        .device_list()
        .find(|d| {
            d.vendor_id() == config.vid
                && d.product_id() == config.pid
                && d.usage_page() == config.usage_page
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No HID device found with VID={:04x} PID={:04x} usage_page={:04x}",
                config.vid,
                config.pid,
                config.usage_page
            )
        })?;

    let device = info.open_device(&api)?;

    let (message_type, body) = match arguments.command {
        Command::Time => (
            MessageType::DateTime,
            postcard::to_allocvec(&date_time::solar_times(
                config.lat,
                config.lon,
                &config.timezone,
                None,
            )?)?,
        ),
        Command::Wake => (
            MessageType::WakeDisplays,
            postcard::to_allocvec(&hid_bridge::Empty {})?,
        ),
        Command::Heap => (
            MessageType::HeapUsage,
            postcard::to_allocvec(&hid_bridge::Empty {})?,
        ),
        Command::DebugOn => (
            MessageType::ToggleDebug,
            postcard::to_allocvec(&hid_bridge::BoolValue { value: true })?,
        ),
        Command::DebugOff => (
            MessageType::ToggleDebug,
            postcard::to_allocvec(&hid_bridge::BoolValue { value: false })?,
        ),
        Command::QueryFrameTime => (
            MessageType::SetFrameTime,
            postcard::to_allocvec(&hid_bridge::U32Value { value: 0 })?,
        ),
        Command::FrameTime { time } => (
            MessageType::SetFrameTime,
            postcard::to_allocvec(&hid_bridge::U32Value { value: time })?,
        ),
        Command::QueryRgbHsv => (
            MessageType::QueryRgbHsv,
            postcard::to_allocvec(&hid_bridge::Empty {})?,
        ),
        Command::SetRgbHsv { h, s, v, save } => (
            MessageType::SetRgbHsv,
            postcard::to_allocvec(&hid_bridge::HsvValue {
                h,
                s,
                v,
                flag: save,
            })?,
        ),
        Command::DisplayBrightness { level, save } => (
            MessageType::SetDisplayBrightness,
            postcard::to_allocvec(&hid_bridge::U8ValueWithFlag {
                value: level,
                flag: save,
            })?,
        ),
    };

    println!("building header for {message_type:?}");
    let header = postcard::to_allocvec(&hid_bridge::MessageHeader::new(
        message_type,
        body.len() as u8,
    ))?;

    println!(
        "building packet for {message_type:?} (header = {}, body = {})",
        header.len(),
        body.len()
    );

    let mut payload = vec![0u8; 32];

    println!("payload length = {}", payload.len());
    payload[0..5].copy_from_slice(&header[..]);
    payload[5..body.len() + 5].copy_from_slice(&body[..]);

    print_packet("→ outgoing", &payload);

    match device.write(&payload) {
        Ok(n) => println!(
            "→ packet of size {} sent ({n} bytes written)",
            5 + body.len()
        ),
        Err(e) => eprintln!("  write error: {e}"),
    }

    std::thread::sleep(Duration::from_millis(10));

    let timeout_ms = 500;

    match device.read_timeout(&mut payload, timeout_ms) {
        Ok(0) => println!("  (no reply within {timeout_ms}ms)"),
        Ok(n) => print_packet("← received", &payload[..n]),
        Err(e) => eprintln!("  read error: {e}"),
    }

    Ok(())
}
