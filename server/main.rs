use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

//use core::time::Duration;


//use manticore::crypto::ring;
//use manticore::hardware::fake;
//use manticore::protocol::capabilities::BusRole;
//use manticore::protocol::capabilities::Networking;
//use manticore::protocol::capabilities::RotMode;
//use manticore::protocol::capabilities::Timeouts;
//use manticore::protocol::device_id;
use manticore::protocol::firmware_version;
use manticore::protocol::wire::FromWire;
use manticore::protocol::CommandType;
use manticore::protocol::Header;
//use manticore::server::pa_rot::Options;
//use manticore::server::pa_rot::PaRot;

//use manticore::server::handler::prelude::*;

////

/// Macro to deserialize wire format from an input stream and then run
/// an operation on the deserialized message.
///
/// This macro is a temporary workaround some Serde limiatations.
///
/// Arguments:
/// * `input`: Identifier for input stream
/// * `body`: A "generic" closure to run with the results of the parse.
#[allow(unused_macros)]
macro_rules! read_wire_and_operate {
    ($input:ident, $body:expr) => {
        let mut input = $input;
        let mut read_buf = Vec::new();
        input
            .read_to_end(&mut read_buf)
            .expect("couldn't read from file");

        let mut arena = vec![0u8; 1024];
        let arena = BumpArena::new(&mut arena);

        let mut read_buf_slice = read_buf.as_slice();
        let header = Header::from_wire(&mut read_buf_slice, &arena)
            .expect("failed to read header");
        match (header.is_request, header.command) {
            (true, CommandType::FirmwareVersion) => {
                let message =
                    firmware_version::FirmwareVersionRequest::from_wire(
                        &mut read_buf_slice,
                        &arena,
                    )
                    .expect("failed to read response");
                let body = $body;
                body(message)
            }
            (false, CommandType::FirmwareVersion) => {
                let message =
                    firmware_version::FirmwareVersionResponse::from_wire(
                        &mut read_buf_slice,
                        &arena,
                    )
                    .expect("failed to read response");
                let body = $body;
                body(message)
            }
            _ => panic!("unsupported response type {:?}", header.command),
        }
    };
}

////

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        handle_connection(stream.unwrap());
    }
}

fn handle_connection(mut stream: TcpStream) {

    // read_wire_and_operate!(stream, |msg| {
    //     println!("msg={:?}", msg)
    // });

    // const DEVICE_ID: device_id::DeviceIdentifier =
    //     device_id::DeviceIdentifier {
    //         vendor_id: 1,
    //         device_id: 2,
    //         subsys_vendor_id: 3,
    //         subsys_id: 4,
    //     };
    // const NETWORKING: Networking = Networking {
    //     max_message_size: 1024,
    //     max_packet_size: 256,
    //     mode: RotMode::Platform,
    //     roles: BusRole::HOST,
    // };
    // const TIMEOUTS: Timeouts = Timeouts {
    //     regular: Duration::from_millis(30),
    //     crypto: Duration::from_millis(200),
    // };
    // let identity = fake::Identity::new(b"test version", b"random bits");
    // let reset = fake::Reset::new(0, Duration::from_millis(1));
    // let rsa = ring::rsa::Builder::new();
    // let mut server = PaRot::new(Options {
    //     identity: &identity,
    //     reset: &reset,
    //     rsa: &rsa,
    //     device_id: DEVICE_ID,
    //     networking: NETWORKING,
    //     timeouts: TIMEOUTS,
    // });

    let mut read_buf = [0; 1024];
    stream.read(&mut read_buf).unwrap();

    // FIXME [dann 2021-02-21]: 5 is a magic number!!!
    let header = Header::from_wire(&mut read_buf[..5])
        .expect("failed to read header");
    println!("header={:?}", header);

    assert_eq!(header.command, CommandType::FirmwareVersion);
    assert_eq!(header.is_request, true);

    let request = firmware_version::FirmwareVersionRequest::from_wire(&mut read_buf[5..])
        .expect("failed to read response");
    println!("request={:?}", request);

    assert_eq!(request.index, 0);

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
