use tokio::io::{self, AsyncRead, AsyncReadExt};

enum State {
    ReadingLength { remainging: usize },
    ReadingPayload { remaining: usize, total: usize },
}

pub struct Reader<R> {
    reader: R,
    buffer: Vec<u8>,
    state: State,
}

impl<R> Reader<R>
where
    R: AsyncRead + Unpin,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: vec![0u8; 65535],
            state: State::ReadingLength { remainging: 2 },
        }
    }

    pub async fn read(&mut self) -> io::Result<&[u8]> {
        loop {
            match self.state {
                State::ReadingLength {
                    remainging: remaining,
                } => {
                    let read = self.reader.read(&mut self.buffer[2 - remaining..2]).await?;

                    let remainging = remaining - read;
                    if remainging == 0 {
                        let length = u16::from_be_bytes([self.buffer[0], self.buffer[1]]) as usize;
                        self.state = State::ReadingPayload {
                            remaining: length,
                            total: length,
                        };
                    } else {
                        self.state = State::ReadingLength { remainging };
                    }
                }
                State::ReadingPayload { remaining, total } => {
                    let read = self
                        .reader
                        .read(&mut self.buffer[total - remaining..total])
                        .await?;

                    let remaining = remaining - read;
                    if remaining == 0 {
                        self.state = State::ReadingLength { remainging: 2 };
                        return Ok(&self.buffer[..total]);
                    } else {
                        self.state = State::ReadingPayload { remaining, total };
                    }
                }
            }
        }
    }
}
