use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};

use manticore::io::Cursor;
use manticore::protocol;
use manticore::protocol::CommandType;
use manticore::protocol::Header;
use manticore::protocol::wire::ToWire;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;

    // Construct a trivial manticore message.
    let hdr = Header {
        is_request: true,
        command: CommandType::FirmwareVersion,
    };
    let req = protocol::firmware_version::FirmwareVersionRequest { index: 0 };
    println!("hdr={:?}", hdr);
    println!("req={:?}", req);

    // Send it
    let mut wire_buf = [0; 32];
    let mut wire = Cursor::new(&mut wire_buf);

    hdr.to_wire(&mut wire).expect("failed to write hdr");
    println!("consumed[{}]={:?}", wire.consumed_len(), wire.consumed_bytes());

    req.to_wire(&mut wire).expect("failed to write req");
    println!("consumed[{}]={:?}", wire.consumed_len(), wire.consumed_bytes());

    let request_bytes = wire.take_consumed_bytes();
    stream.write(request_bytes)?;
    stream.shutdown(Shutdown::Write).expect("shutdown call failed");

    // FIXME [dann 2021-02-21]: Read/validate response.
    let mut resp_buf = [0; 32];
    stream.read(&mut resp_buf)?;
    println!("resp={:?}", resp_buf);

    Ok(())
} // the stream is closed here
