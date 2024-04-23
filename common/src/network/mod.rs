use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

use serde::{Deserialize, Serialize};

pub mod game;
pub mod lobby;

pub trait TcpStreamExt {
    fn read_message<T: for<'de> Deserialize<'de>>(
        &mut self,
        timeout: Option<Duration>,
    ) -> anyhow::Result<T>;
    fn write_message<T: Serialize>(&mut self, value: &T) -> anyhow::Result<()>;
}

impl TcpStreamExt for TcpStream {
    fn read_message<T: for<'de> Deserialize<'de>>(
        &mut self,
        timeout: Option<Duration>,
    ) -> anyhow::Result<T> {
        let old_timeout = self.read_timeout().unwrap_or(None);
        self.set_read_timeout(timeout)?;
        let mut len = [0; 4];
        self.read_exact(&mut len)?;
        let len: u32 = u32::from_be_bytes(len);
        println!("Reading {} bytes", len);

        let mut buffer = vec![0; len as _];
        self.read_exact(&mut buffer)?;
        self.set_read_timeout(old_timeout)?;
        Ok(postcard::from_bytes(&buffer)?)
        // Ok(serde_json::from_slice(&buffer)?)
    }

    fn write_message<T: Serialize>(&mut self, value: &T) -> anyhow::Result<()> {
        let bytes = postcard::to_allocvec(value)?;
        // let bytes = serde_json::to_vec(value)?;
        let len = (bytes.len() as u32).to_be_bytes();
        println!("Writing {} bytes", bytes.len());
        self.write_all(&len)?;
        self.write_all(&bytes)?;
        Ok(())
    }
}
