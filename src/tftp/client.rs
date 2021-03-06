//! A Trivial File Transfer (TFTP) protocol client implementation.
//!
//! This module contains the ability to read data from or write data to a remote TFTP server.

use std::borrow::IntoCow;
use std::io;
use std::path::Path;
use std::net::{UdpSocket, SocketAddr, IpAddr};

use packet::{Mode, RequestPacket, DataPacketOctet, AckPacket, ErrorPacket,
             EncodePacket, RawPacket, Error};

static MAX_DATA_SIZE: usize = 512;

//#[derive(Debug, Clone)]
//pub enum ClientError {
    //TftpError(Error, String),
    //IoError(IoError),
//}

//impl ClientError {
    //pub fn from_io(err: IoError) -> ClientError {
        //ClientError::IoError(err)
    //}
//}

//pub type ClientResult<T> = Result<T, ClientError>;

/// A Trivial File Transfer Protocol client.
pub struct Client {
    socket: UdpSocket,
    remote_addr: SocketAddr,
}

impl Client {
    /// Creates a new client and binds an UDP socket.
    pub fn new(remote_addr: SocketAddr) -> io::Result<Client> {
        // FIXME: port should not be hardcoded
        let addr = SocketAddr::new(IpAddr::new_v4(127, 0, 0, 1), 41000);
        UdpSocket::bind(&addr).map(|socket| {
            Client{ socket: socket, remote_addr: remote_addr }
        })
    }

    /// A TFTP read request
    ///
    /// Get a file `path` from the server using a `mode`. Received data is written to
    /// the `writer`.
    pub fn get(&mut self, path: &Path, mode: Mode, writer: &mut io::Write) -> io::Result<()> {
        let mut bufs = vec![vec![0; MAX_DATA_SIZE + 4], vec![0; MAX_DATA_SIZE + 4]];

        let read_request = RequestPacket::read_request(path.to_str().expect("utf-8 path"), mode);
        let encoded = read_request.encode_using(bufs.pop().unwrap());
        try!(self.socket.send_to(encoded.packet_buf(), &self.remote_addr));

        bufs.push(encoded.get_buffer());
        let mut first_packet = true;
        let mut current_id = 1;
        loop {
            // TODO
            //self.socket.set_timeout(Some(1000));
            let mut buf = bufs.pop().unwrap();
            match self.socket.recv_from(buf.as_mut_slice()) {
                Ok((n, from)) => {
                    if first_packet && self.remote_addr.ip() == from.ip() {
                        self.remote_addr = SocketAddr::new(self.remote_addr.ip(), from.port());
                        first_packet = false;
                    }
                    if from != self.remote_addr {
                        bufs.push(buf);
                        continue
                    }
                    let packet = RawPacket::new(buf, n);
                    {
                        match packet.decode::<DataPacketOctet>() {
                            Some(data_packet) => {
                                if current_id == data_packet.block_id() {
                                    try!(writer.write_all(data_packet.data()));
                                    let ack = AckPacket::new(data_packet.block_id());
                                    let buf = bufs.pop().unwrap();
                                    let encoded = ack.encode_using(buf);
                                    try!(self.socket.send_to(encoded.packet_buf(), &self.remote_addr));
                                    if data_packet.data().len() < MAX_DATA_SIZE {
                                        println!("Transfer complete");
                                        break;
                                    }
                                    bufs.push(encoded.get_buffer());
                                    current_id += 1;
                                } else {
                                    println!("Unexpected packet id: got={}, expected={}",
                                           data_packet.block_id(), current_id);
                                }
                            }
                            None => {
                                match packet.decode::<ErrorPacket>() {
                                    Some(err) => {
                                        return Err(io::Error::new(io::ErrorKind::Other, "todo", None))
                                     }
                                    None => {
                                        //let opcode = packet.opcode().map(|o| format!("{}", o))
                                                                    //.unwrap_or("Unknown".to_string());
                                        //println!("Unexpected packet type: iopcode={}", opcode)
                                        println!("Unexpected packet type");
                                    }

                                }
                            }
                        }
                    }
                    bufs.push(packet.get_buffer());
                }
                Err(e) => {
                    bufs.push(buf);
                    return Err(e);
                }
            }
        }
        return Ok(())
    }

    /// A TFTP write request
    ///
    /// Put a file `path` to the server using a `mode`.
    pub fn put(&mut self, _path: &Path, _mode: Mode, _reader: &mut io::Read) -> io::Result<()> {
        //let mut bufs = Vec::from_fn(2, |_| Vec::from_elem(MAX_DATA_SIZE + 4, 0));
        //let mut read_buffer = Vec::from_elem(MAX_DATA_SIZE, 0);

        //let read_request = RequestPacket::write_request(path.as_str().expect("utf-8 path"), mode);
        //let encoded = read_request.encode_using(bufs.pop().unwrap());
        //try!(self.socket.send_to(encoded.packet_buf(), self.remote_addr));

        //bufs.push(encoded.get_buffer());
        //let mut first_packet = true;
        //let mut last_packet = false;
        //let mut current_id = 0;
        //loop {
            //let mut buf = bufs.pop().unwrap();
            //match self.socket.recv_from(buf.as_mut_slice()) {
                //Ok((n, from)) => {
                    //if first_packet && self.remote_addr.ip == from.ip {
                        //self.remote_addr.port = from.port;
                        //first_packet = false;
                    //}
                    //if from != self.remote_addr {
                        //bufs.push(buf);
                        //continue
                    //}
                    //let packet = RawPacket::new(buf, n);
                    //{
                        //let ack_packet: Option<AckPacket> = packet.decode();
                        //match ack_packet {
                            //Some(dp) => {
                                //if current_id == dp.block_id() {
                                    //if last_packet {
                                        //println!("done");
                                        //break
                                    //}
                                    //let bytes_read = try!(self.read_block(reader, read_buffer.as_mut_slice()));
                                    //if bytes_read < MAX_DATA_SIZE {
                                        //last_packet = true;
                                    //}
                                    //let ack = DataPacketOctet::from_slice(dp.block_id() + 1, read_buffer.slice_to(bytes_read));
                                    //let buf = bufs.pop().unwrap();
                                    //let encoded = ack.encode_using(buf);
                                    //try!(self.socket.send_to(encoded.packet_buf(), self.remote_addr));
                                    //bufs.push(encoded.get_buffer());
                                    //current_id += 1;
                                //} else {
                                    //println!("wrong packet id");
                                //}
                            //}
                            //None => fail!("not a data packet")
                        //}
                    //}
                    //bufs.push(packet.get_buffer());
                //}
                //Err(ref e) => fail!("error = {}", e)
            //}
        //}
        return Ok(())
    }

    //fn read_block(&mut self, reader: &mut Reader, buf: &mut [u8]) -> IoResult<usize> {
        //reader.read(buf).or_else(|e| {
            //if e.kind == old_io::IoErrorKind::EndOfFile {
                //Ok(0usize)
            //} else {
                //Err(e)
            //}
        //})
    //}
}
