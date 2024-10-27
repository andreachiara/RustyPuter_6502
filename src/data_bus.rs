use core::fmt;
use std::{error::Error, fmt::{write}, iter::FlatMap, io};

use crate::memory;

const MAX_BUS_SIZE: usize = 0x10000;

struct Endpoint {
    begin: usize,
    size: usize,
    data: [u8; MAX_BUS_SIZE],
    read_cb: fn(usize) -> u8,
    write_cb: fn(usize, u8)
}

impl Endpoint {
    fn new(begin: usize, size: usize, read_cb: fn(usize)->u8, write_cb: fn(usize, u8)) -> Endpoint {
        let new_ep = Endpoint {
            begin: begin,
            size: size,
            data: [0; MAX_BUS_SIZE],
            read_cb: read_cb,
            write_cb: write_cb
        };

        return new_ep;
    }

    fn read(&self, address: usize) -> u8 {
        return (self.read_cb)(address);
    }

    fn write(&mut self, address: usize, byte_to_write: u8) {
        return (self.write_cb)(address, byte_to_write);
    }
}
struct DataBus {
    endpoints: Vec<Endpoint>,
}

impl DataBus {
    fn get_endpoint(&mut self, address: usize) -> Result<&mut Endpoint, io::Error> {
        for ep in self.endpoints.iter_mut() {
            if ep.begin + ep.size < address {
                continue;
            }
            return Ok(ep);
        }
        return Err(io::Error::new(io::ErrorKind::NotFound, "the address was not found on the bus"));
    }

    fn read_byte(&mut self, address: usize) -> Result<u8, io::Error> {
        let ep = self.get_endpoint(address)?;

        let rel_addr = address - ep.begin;
        return Ok(ep.read(rel_addr));
    }

    fn write_byte(&mut self, address: usize, byte_w: u8) -> Result<(),io::Error> {
        let mut ep = self.get_endpoint(address)?;

        let rel_addr = address - ep.begin;

        ep.write(rel_addr, byte_w);

        return Ok(());
    }
}



