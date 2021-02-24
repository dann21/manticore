use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};

use manticore::io::Cursor;
use manticore::protocol;
use manticore::protocol::CommandType;
use manticore::protocol::Header;
use manticore::protocol::spi_payload::SpiHeader;
use manticore::protocol::spi_payload::SpiContentType;
use manticore::protocol::wire::ToWire;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;

    // Construct a trivial manticore message.
    let mmsg_hdr = Header {
        is_request: true,
        command: CommandType::FirmwareVersion,
    };
    let mmsg_req = protocol::firmware_version::FirmwareVersionRequest { index: 0 };
    println!("mmsg_hdr={:?}", mmsg_hdr);
    println!("mmsg_req={:?}", mmsg_req);

    // Obtain its size.
    let mut mmsg_buf = [0; 32];
    let mut mmsg_cursor = Cursor::new(&mut mmsg_buf);
    mmsg_hdr.to_wire(&mut mmsg_cursor).expect("failed to write mmsg_hdr");
    println!("consumed[{}]={:?}", mmsg_cursor.consumed_len(), mmsg_cursor.consumed_bytes());
    mmsg_req.to_wire(&mut mmsg_cursor).expect("failed to write mmsg_req");
    println!("consumed[{}]={:?}", mmsg_cursor.consumed_len(), mmsg_cursor.consumed_bytes());
    let mmsg_len = mmsg_cursor.consumed_len() as u16;
    println!("mmsg_bytes[{}]={:?}", mmsg_len, mmsg_cursor.consumed_bytes());

    // Send it off.
    let spi_hdr = SpiHeader {
        content_type: SpiContentType::Manticore,
        content_len: mmsg_len
    };
    let mut wire_buf = [0; 32];
    let mut wire = Cursor::new(&mut wire_buf);
    spi_hdr.to_wire(&mut wire).expect("failed to write spi_hdr");
    mmsg_hdr.to_wire(&mut wire).expect("failed to write mmsg_hdr");
    mmsg_req.to_wire(&mut wire).expect("failed to write mmsg_req");

    let spi_msg_bytes = wire.take_consumed_bytes();
    println!("spi_msg_bytes[{}]={:?}", spi_msg_bytes.len(), spi_msg_bytes);

    stream.write(spi_msg_bytes)?;
    stream.shutdown(Shutdown::Write).expect("shutdown call failed");

    // FIXME [dann 2021-02-21]: Read/validate response.
    let mut resp_buf = [0; 32];
    stream.read(&mut resp_buf)?;
    println!("resp={:?}", resp_buf);

    Ok(())
} // the stream is closed here
