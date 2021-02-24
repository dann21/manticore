use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use core::time::Duration;

use manticore::crypto::ring;
use manticore::hardware::fake;
use manticore::protocol::capabilities::BusRole;
use manticore::protocol::capabilities::Networking;
use manticore::protocol::capabilities::RotMode;
use manticore::protocol::capabilities::Timeouts;
use manticore::protocol::device_id;
use manticore::protocol::wire::FromWire;
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
    let _server = PaRot::new(Options {
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

    match spi_hdr.content_type {
        SpiContentType::Manticore => {
            let mut data_buf : Vec<u8> = vec![0; spi_hdr.content_len as usize];
            stream.read(&mut data_buf).unwrap();
            println!("data_buf={:?}", data_buf);

            // Eventually call process_request()...
            ()
        }
        _ => ()
    }

    // let mut arena = [0; 64];
    // //let mut arena = BumpArena::new(&mut arena);
    // let arena = BumpArena::new(&mut arena);

    // // FIXME [dann 2021-02-21]: 5 is a magic number!!!
    // let header = Header::from_wire(&mut read_buf[..5], &arena)
    //     .expect("failed to read header");
    // println!("header={:?}", header);

    // assert_eq!(header.command, CommandType::FirmwareVersion);
    // assert_eq!(header.is_request, true);

    // let request = firmware_version::FirmwareVersionRequest::from_wire(
    //     &mut read_buf[5..], &arena
    // ).expect("failed to read response");
    // println!("request={:?}", request);

    // assert_eq!(request.index, 0);

    // let get = b"GET / HTTP/1.1\r\n";

    // let (status_line, filename) = if buffer.starts_with(get) {
    //     ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    // } else {
    //     ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    // };

    // let contents = fs::read_to_string(filename).unwrap();

    // let response = format!("{}{}", status_line, contents);

    // stream.write(response.as_bytes()).unwrap();
    // stream.flush().unwrap();
}
