use std::{array, borrow::Cow};

pub const COMMAND_SIZE: usize = 13;
pub const MAX_PAYLOAD: usize = 0x2000;
const TEXT_CAPACITY: usize = COMMAND_SIZE - 1;

#[must_use]
fn command(text: &str) -> [u8; COMMAND_SIZE] {
    let bytes = text.as_bytes();
    let kept = bytes
        .get(..bytes.len().min(TEXT_CAPACITY))
        .unwrap_or_default();
    array::from_fn(|index| kept.get(index).copied().unwrap_or(0))
}

#[must_use]
pub fn size_command(total: u32) -> [u8; COMMAND_SIZE] {
    command(&format!("s 0x{total:08x}"))
}

#[must_use]
pub fn block_command(length: u32) -> [u8; COMMAND_SIZE] {
    command(&format!("w 0x{length:08x}"))
}

#[must_use]
pub fn quit_command() -> [u8; COMMAND_SIZE] {
    command("q")
}

#[must_use]
pub fn is_ack(reply: &[u8]) -> bool {
    reply.starts_with(b"ok")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step<'file> {
    Write(Cow<'file, [u8]>),
    ReadAck,
}

pub struct Sender<'file> {
    file: &'file [u8],
    offset: usize,
    phase: Phase,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Size,
    BlockHeader,
    BlockBody,
    Ack,
    Quit,
    Done,
}

impl<'file> Sender<'file> {
    #[must_use]
    pub const fn new(file: &'file [u8]) -> Self {
        Self {
            file,
            offset: 0,
            phase: Phase::Size,
        }
    }

    #[must_use]
    pub const fn sent(&self) -> usize {
        self.offset
    }

    fn block(&self) -> &'file [u8] {
        let end = self.offset.saturating_add(MAX_PAYLOAD).min(self.file.len());
        self.file.get(self.offset..end).unwrap_or_default()
    }
}

impl<'file> Iterator for Sender<'file> {
    type Item = Step<'file>;

    fn next(&mut self) -> Option<Step<'file>> {
        let step = match self.phase {
            Phase::Size => {
                self.phase = if self.file.is_empty() {
                    Phase::Quit
                } else {
                    Phase::BlockHeader
                };
                Step::Write(Cow::Owned(
                    size_command(u32::try_from(self.file.len()).unwrap_or(u32::MAX)).to_vec(),
                ))
            }
            Phase::BlockHeader => {
                self.phase = Phase::BlockBody;
                let length = u32::try_from(self.block().len()).unwrap_or(u32::MAX);
                Step::Write(Cow::Owned(block_command(length).to_vec()))
            }
            Phase::BlockBody => {
                self.phase = Phase::Ack;
                let block = self.block();
                self.offset = self.offset.saturating_add(block.len());
                Step::Write(Cow::Borrowed(block))
            }
            Phase::Ack => {
                self.phase = if self.offset >= self.file.len() {
                    Phase::Quit
                } else {
                    Phase::BlockHeader
                };
                Step::ReadAck
            }
            Phase::Quit => {
                self.phase = Phase::Done;
                Step::Write(Cow::Owned(quit_command().to_vec()))
            }
            Phase::Done => return None,
        };
        Some(step)
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_PAYLOAD, Sender, Step, block_command, is_ack, quit_command, size_command};

    #[test]
    fn commands_match_the_spec_bytes() {
        assert_eq!(size_command(0x0004_B000), *b"s 0x0004b000\0");
        assert_eq!(block_command(0x2000), *b"w 0x00002000\0");
        assert_eq!(quit_command(), *b"q\0\0\0\0\0\0\0\0\0\0\0\0");
        assert!(is_ack(b"ok\0\0\0\0\0\0\0\0\0\0\0"));
    }

    fn writes(file: &[u8]) -> Vec<usize> {
        Sender::new(file)
            .filter_map(|step| match step {
                Step::Write(bytes) => Some(bytes.len()),
                Step::ReadAck => None,
            })
            .collect()
    }

    #[test]
    fn an_empty_file_sends_only_size_and_quit() {
        let steps: Vec<Step<'_>> = Sender::new(&[]).collect();
        assert_eq!(
            steps,
            [
                Step::Write(size_command(0).to_vec().into()),
                Step::Write(quit_command().to_vec().into())
            ]
        );
    }

    #[test]
    fn a_full_block_fits_in_one_write() {
        assert_eq!(writes(&[0; MAX_PAYLOAD]), [13, 13, MAX_PAYLOAD, 13]);
        assert_eq!(
            writes(&[0; MAX_PAYLOAD + 1]),
            [13, 13, MAX_PAYLOAD, 13, 1, 13]
        );
    }

    #[test]
    fn a_307200_byte_file_takes_38_blocks() {
        let file = vec![7_u8; 307_200];
        let steps: Vec<Step<'_>> = Sender::new(&file).collect();
        let headers = steps
            .iter()
            .filter(|step| matches!(step, Step::Write(bytes) if bytes.starts_with(b"w ")))
            .count();
        let acks = steps.iter().filter(|step| **step == Step::ReadAck).count();
        let quits = steps
            .iter()
            .filter(|step| matches!(step, Step::Write(bytes) if bytes.starts_with(b"q")))
            .count();
        assert_eq!((headers, acks, quits), (38, 38, 1));
        let payload: usize = steps
            .iter()
            .filter_map(|step| match step {
                Step::Write(bytes) if bytes.len() != 13 => Some(bytes.len()),
                _ => None,
            })
            .sum();
        assert_eq!(payload, 307_200);
        assert!(matches!(steps.last(), Some(Step::Write(bytes)) if bytes.starts_with(b"q")));
    }
}
