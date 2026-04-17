#![cfg(windows)]

use std::fs::OpenOptions;
use std::io::{self, Read, Write};

use crate::agent::AgentBackend;

pub struct NamedPipeBackend {
    reader: std::fs::File,
    writer: std::fs::File,
}

impl NamedPipeBackend {
    pub fn connect(pipe_name: &str) -> io::Result<Self> {
        let writer = OpenOptions::new().read(true).write(true).open(pipe_name)?;
        let reader = writer.try_clone()?;
        Ok(Self { reader, writer })
    }
}

impl AgentBackend for NamedPipeBackend {
    fn roundtrip(&mut self, request: &[u8]) -> io::Result<Vec<u8>> {
        self.writer.write_all(request)?;
        self.writer.flush()?;

        let mut len_buf = [0_u8; 4];
        self.reader.read_exact(&mut len_buf)?;
        let payload_len = u32::from_be_bytes(len_buf) as usize;
        let mut response = Vec::with_capacity(4 + payload_len);
        response.extend_from_slice(&len_buf);
        response.resize(4 + payload_len, 0);
        self.reader.read_exact(&mut response[4..])?;

        Ok(response)
    }
}
