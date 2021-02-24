use std::convert::TryFrom;
use std::io::prelude::*;
use std::net::{Shutdown, TcpStream, TcpListener};

use core::time::Duration;

use manticore::crypto::ring;
use manticore::hardware::fake;
use manticore::io::Cursor;
use manticore::protocol::capabilities::BusRole;
use manticore::protocol::capabilities::Networking;
use manticore::protocol::capabilities::RotMode;
use manticore::protocol::capabilities::Timeouts;
use manticore::protocol::device_id;
use manticore::protocol::wire::FromWire;
use manticore::protocol::wire::FromWireError;
use manticore::protocol::wire::ToWire;
use manticore::protocol::spi_payload::SpiHeader;
use manticore::protocol::spi_payload::SPI_HEADER_LEN;
use manticore::protocol::spi_payload::SpiContentType;
use manticore::server::pa_rot::Options;
use manticore::server::pa_rot::PaRot;

////

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        handle_connection(stream.unwrap());
    }
}

fn handle_connection(mut stream: TcpStream) {

    // Dummy up a server.
    const DEVICE_ID: device_id::DeviceIdentifier =
        device_id::DeviceIdentifier {
            vendor_id: 1,
            device_id: 2,
            subsys_vendor_id: 3,
            subsys_id: 4,
        };
    const NETWORKING: Networking = Networking {
        max_message_size: 1024,
        max_packet_size: 256,
        mode: RotMode::Platform,
        roles: BusRole::HOST,
    };
    const TIMEOUTS: Timeouts = Timeouts {
        regular: Duration::from_millis(30),
        crypto: Duration::from_millis(200),
    };
    let identity = fake::Identity::new(b"test version", b"random bits");
    let reset = fake::Reset::new(0, Duration::from_millis(1));
    let rsa = ring::rsa::Builder::new();
    let mut server = PaRot::new(Options {
        identity: &identity,
        reset: &reset,
        rsa: &rsa,
        device_id: DEVICE_ID,
        networking: NETWORKING,
        timeouts: TIMEOUTS,
    });

    // Grab the SPI header bytes off the wire.
    let mut spi_hdr_buf: Vec<u8> = vec![0; SPI_HEADER_LEN];
    stream.read(&mut spi_hdr_buf).unwrap();
    println!("spi_hdr_buf={:?}", spi_hdr_buf);
    let mut spi_hdr_buf = spi_hdr_buf.as_slice();
    let spi_hdr = SpiHeader::from_wire(&mut spi_hdr_buf).unwrap();

    // Process the SPI message.
    match spi_hdr.content_type {
        SpiContentType::Manticore => {
            // Pull manticore-msg off the wire...
            let mut mmsg_buf : Vec<u8> = vec![0; spi_hdr.content_len as usize];
            stream.read(&mut mmsg_buf).unwrap();
            println!("mmsg_buf[{}]={:?}", mmsg_buf.len(), mmsg_buf);
            let mut mmsg_buf = mmsg_buf.as_slice();

            // FIXME [dann 2021-02-24]: 512 is a Magic Number!
            let mut resp_buf : [u8; 512] = [0xff; 512];
            let mmsg_resp_len : u16;

            // Process the request...
            {
                // Need to process the request first (to obtain
                // content-len) before we can construct the header.
                let mut mmsg_resp_cursor = Cursor::new(&mut resp_buf[SPI_HEADER_LEN..]);
                server.process_request(&mut mmsg_buf, &mut mmsg_resp_cursor).unwrap();
                mmsg_resp_len = u16::try_from(mmsg_resp_cursor.consumed_len())
                    .map_err(|_| FromWireError::OutOfRange).unwrap();
                println!("mmsg_resp[{}]={:?}", mmsg_resp_len, &resp_buf[SPI_HEADER_LEN..SPI_HEADER_LEN + (mmsg_resp_len as usize)]);
            }

            // Serialize the SPI header into resp_buf, up front...
            let resp_hdr = SpiHeader {
                content_type: SpiContentType::Manticore,
                content_len: mmsg_resp_len,
            };
            let spi_resp_hdr_len : u16;
            {
                let mut spi_resp_hdr_cursor = Cursor::new(&mut resp_buf[..SPI_HEADER_LEN]);
                resp_hdr.to_wire(&mut spi_resp_hdr_cursor).expect("failed to write spi_resp_hdr");
                spi_resp_hdr_len = u16::try_from(spi_resp_hdr_cursor.consumed_len())
                    .map_err(|_| FromWireError::OutOfRange).unwrap();
            }

            // Send off the response
            let spi_resp_len = spi_resp_hdr_len+mmsg_resp_len;
            println!("resp_buf[{}]={:?}", spi_resp_len, &resp_buf[..spi_resp_len as usize]);
            stream.write(&resp_buf[..spi_resp_len as usize]).unwrap();
        }
        _ => ()
    }
    stream.shutdown(Shutdown::Write).expect("shutdown call failed");
}
