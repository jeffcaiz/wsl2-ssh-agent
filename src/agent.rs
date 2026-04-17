use std::io::{self, Read, Write};

const AGENT_MAX_MESSAGE_LENGTH: usize = 1024 * 1024;

pub trait AgentBackend {
    fn roundtrip(&mut self, request: &[u8]) -> io::Result<Vec<u8>>;
}

pub fn serve_stdio(mut backend: impl AgentBackend) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    loop {
        let Some(request) = read_message(&mut reader)? else {
            return Ok(());
        };

        let response = backend.roundtrip(&request)?;
        writer.write_all(&response)?;
        writer.flush()?;
    }
}

fn read_message(reader: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut len_buf = [0_u8; 4];
    match reader.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(err) => return Err(err),
    }

    let payload_len = u32::from_be_bytes(len_buf) as usize;
    if payload_len > AGENT_MAX_MESSAGE_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("agent request too large: {payload_len} bytes"),
        ));
    }

    let mut message = Vec::with_capacity(4 + payload_len);
    message.extend_from_slice(&len_buf);
    message.resize(4 + payload_len, 0);
    reader.read_exact(&mut message[4..])?;

    Ok(Some(message))
}
