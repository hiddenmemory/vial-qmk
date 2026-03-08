use clap::{Parser, Subcommand};
use hid_bridge::MessageType;
use hidapi::HidApi;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Arguments {
    /// Vendor ID in hex (e.g. 04d8) or decimal (e.g. 1240). Required for all commands.
    #[arg(short, long)]
    vid: String,

    /// Product ID in hex (e.g. 003f) or decimal (e.g. 63). Required for all commands.
    #[arg(short, long)]
    pid: String,

    /// Usage Page in hex (e.g. 003f) or decimal (e.g. 63). Required for all commands.
    #[arg(short, long)]
    usage_page: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Time,
    Wake,
}

fn parse_id(s: &str) -> anyhow::Result<u16> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u16::from_str_radix(hex, 16).map_err(|e| anyhow::anyhow!("bad hex '{s}': {e}"))
    } else {
        s.parse::<u16>()
            .map_err(|e| anyhow::anyhow!("bad decimal '{s}': {e}"))
    }
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

fn seconds_since_midnight() -> u32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    (now.as_secs() % 86_400) as u32
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();

    let vid = parse_id(&arguments.vid)?;
    let pid = parse_id(&arguments.pid)?;
    let usage_page = parse_id(&arguments.usage_page)?;

    let api = HidApi::new()?;

    let info = api
        .device_list()
        .find(|d| d.vendor_id() == vid && d.product_id() == pid && d.usage_page() == usage_page)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No HID device found with VID={:04x} PID={:04x} usage_page={:04x}",
                vid,
                pid,
                usage_page
            )
        })?;

    let device = info.open_device(&api)?;

    let (message_type, body) = match arguments.command {
        Command::Time => (
            MessageType::DateTime,
            postcard::to_allocvec(&hid_bridge::DateTime {
                seconds_since_midnight: seconds_since_midnight(),
            })?,
        ),
        Command::Wake => (
            MessageType::WakeDisplays,
            postcard::to_allocvec(&hid_bridge::Empty {})?,
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
