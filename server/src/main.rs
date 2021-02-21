// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

//! `manticore-server` is a toy manticore server that may eventually
//! be used by unit tests.

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use core::time::Duration;

use manticore::crypto::ring;
use manticore::hardware::fake;
use manticore::io::write::StdWrite;
use manticore::io::write::Write;
use manticore::mem::BumpArena;
use manticore::net::InMemHost;
use manticore::protocol::capabilities::BusRole;
use manticore::protocol::capabilities::Networking;
use manticore::protocol::capabilities::RotMode;
use manticore::protocol::capabilities::Timeouts;
use manticore::protocol::device_id;
use manticore::protocol::spi_payload::SpiContentType;
use manticore::protocol::spi_payload::SpiHeader;
use manticore::protocol::spi_payload::SPI_HEADER_LEN;
use manticore::protocol::wire::FromWire;
use manticore::protocol::wire::ToWire;
use manticore::protocol::Header;
use manticore::protocol::HEADER_LEN;
use manticore::server;
use manticore::server::pa_rot::Options;
use manticore::server::pa_rot::PaRot;

////

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        handle_connection(stream.unwrap()).unwrap();
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), server::Error> {
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
    let identity = fake::Identity::new(
        b"test version",
        &[(0, b"random_version")],
        b"random bits",
    );
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

    // On-stack scratch space for random allocations.
    // FIXME [dann 2021-03-16]: 1024 is a Magic Number!
    let mut scratch = [0u8; 1024];
    let arena = BumpArena::new(&mut scratch[..]);

    // Grab SPI header bytes off the wire.
    let mut spi_hdr_bytes_buf = [0u8; SPI_HEADER_LEN];
    stream
        .read_exact(&mut spi_hdr_bytes_buf)
        .expect("SPI header read failed!");
    println!("spi_hdr_bytes_buf={:?}", spi_hdr_bytes_buf);

    // Deserialize SPI header.
    let spi_hdr = SpiHeader::from_wire(&mut spi_hdr_bytes_buf[..], &arena)?;

    // Process SPI message.
    #[allow(clippy::single_match)]
    match spi_hdr.content_type {
        SpiContentType::Manticore => {
            // Grab manticore hdr+payload bytes off the wire...
            let mut mmsg_buf = vec![0u8; spi_hdr.content_len as usize];
            stream
                .read_exact(&mut mmsg_buf)
                .expect("manticore hdr+payload read failed!");
            println!("mmsg_buf[{}]={:?}", mmsg_buf.len(), mmsg_buf);

            // Parse manticore hdr.
            let mmsg_hdr = Header::from_wire(&mmsg_buf[..HEADER_LEN], &arena)?;
            assert!(mmsg_hdr.is_request);

            // On-stack space for assembling manticore response payload.
            // FIXME [dann 2021-02-24]: 512 is a Magic Number!
            let mut mresp_payload_buf = [0xffu8; 512];

            // InMemHost is a simple HostPort implementation.
            let mut host = InMemHost::new(&mut mresp_payload_buf);

            // Queue up manticore request (hdr+payload) for processing.
            host.request(mmsg_hdr, &mmsg_buf[HEADER_LEN..]);

            // Handle the request.  Response is left in mresp_payload_buf.
            server.process_request(&mut host, &arena)?;
            let (mresp_hdr, mresp_payload) = host.response().unwrap();

            // Wraps manticore response (hdr+payload).
            let spi_resp_hdr = SpiHeader {
                content_type: SpiContentType::Manticore,
                content_len: (HEADER_LEN + mresp_payload.len()) as u16,
            };

            println!("spi_resp_hdr={:?}", spi_resp_hdr);
            println!("mresp_hdr={:?}", mresp_hdr);
            println!(
                "mresp_payload[{}]={:?}",
                mresp_payload.len(),
                mresp_payload
            );

            // Send response (spi-hdr; mresp-hdr; mresp-payload).
            let mut write = StdWrite(stream);
            spi_resp_hdr.to_wire(&mut write)?;
            mresp_hdr.to_wire(&mut write)?;
            write
                .write_bytes(mresp_payload)
                .expect("response: write_bytes failed!");

            println!("done!");
        }
        _ => (),
    }
    Ok(())
}
