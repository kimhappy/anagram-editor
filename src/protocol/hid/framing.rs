pub const REPORT_SIZE: usize = 64;
const CHUNK: usize = REPORT_SIZE - 1;
const LEADING_BYTE: u8 = 0x01;
const START: &[u8] = b"DSTART";
const END: &[u8] = b"DEND";
const STUFFING: u8 = 0x00;

#[must_use]
pub fn frame_request(json: &str) -> Vec<[u8; REPORT_SIZE]> {
    let body: Vec<u8> = [START, &stuff_sentinels(json.as_bytes()), END].concat();
    body.chunks(CHUNK)
        .map(Some)
        .chain(std::iter::once(None))
        .map(|chunk| {
            let mut report = [0; REPORT_SIZE];
            report[0] = LEADING_BYTE;
            if let Some(chunk) = chunk
                && let Some(target) = report.get_mut(1..=chunk.len())
            {
                target.copy_from_slice(chunk);
            }
            report
        })
        .collect()
}

fn stuff_sentinels(payload: &[u8]) -> Vec<u8> {
    let opens_sentinel = |index: usize| {
        let rest = payload.get(index..).unwrap_or_default();
        rest.starts_with(START) || rest.starts_with(END)
    };
    payload
        .iter()
        .enumerate()
        .flat_map(|(index, &byte)| {
            let stuffing = opens_sentinel(index).then_some(STUFFING);
            std::iter::once(byte).chain(stuffing)
        })
        .collect()
}

#[derive(Debug, Default)]
pub struct Reassembler {
    buffer: Vec<u8>,
}

impl Reassembler {
    pub fn push(&mut self, report: &[u8]) -> Option<String> {
        let body = report.strip_prefix(&[LEADING_BYTE]).unwrap_or(report);
        self.buffer.extend_from_slice(body);
        let Some(start) = find(&self.buffer, START) else {
            let keep = self.buffer.len().saturating_sub(START.len());
            self.buffer.drain(..keep);
            return None;
        };
        let payload_start = start + START.len();
        let end = payload_start + find(self.buffer.get(payload_start..)?, END)?;
        let document: Vec<u8> = self
            .buffer
            .get(payload_start..end)?
            .iter()
            .copied()
            .filter(|&byte| byte != STUFFING && byte != LEADING_BYTE)
            .collect();
        self.buffer.drain(..end + END.len());
        Some(String::from_utf8_lossy(&document).into_owned())
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::{LEADING_BYTE, REPORT_SIZE, Reassembler, frame_request};

    const REQUEST: &str = r#"{"action":"fetch_firmware_version","payload":{}}"#;

    #[test]
    fn the_worked_example_takes_two_reports() {
        let reports = frame_request(REQUEST);
        assert_eq!(reports.len(), 2);
        let first = reports.first().expect("first report");
        assert_eq!(first[0], 0x01);
        assert_eq!(&first[1..7], b"DSTART");
        assert_eq!(&first[55..59], b"DEND");
        assert!(first[59..].iter().all(|&byte| byte == 0));
        let terminator = reports.get(1).expect("terminator");
        assert_eq!(terminator[0], 0x01);
        assert!(terminator[1..].iter().all(|&byte| byte == 0));
    }

    #[test]
    fn sentinels_inside_a_request_are_stuffed() {
        let json = r#"{"action":"x","payload":{"name":"ADDEND","m":"DSTART"}}"#;
        let body: Vec<u8> = frame_request(json)
            .iter()
            .flat_map(|report| report.get(1..).unwrap_or_default().to_vec())
            .collect();
        let expected: &[u8] =
            b"DSTART{\"action\":\"x\",\"payload\":{\"name\":\"ADD\x00END\",\"m\":\"D\x00START\"}}DEND";
        assert_eq!(body.get(..expected.len()), Some(expected));
        let mut reassembler = Reassembler::default();
        let replayed: Vec<u8> = [&[LEADING_BYTE], expected].concat();
        assert_eq!(reassembler.push(&replayed), Some(json.to_owned()));
    }

    #[test]
    fn long_requests_split_at_63_bytes() {
        let json = format!(
            r#"{{"action":"x","payload":{{"text":"{}"}}}}"#,
            "a".repeat(200)
        );
        let reports = frame_request(&json);
        let body_len = json.len() + 10;
        assert_eq!(reports.len(), body_len.div_ceil(63) + 1);
        assert!(reports.iter().all(|report| report.len() == REPORT_SIZE));
    }

    #[test]
    fn reassembly_strips_padding_and_waits_for_the_end() {
        let mut reassembler = Reassembler::default();
        let report = |chunk: &[u8]| {
            let mut report = vec![0x01_u8];
            report.extend_from_slice(chunk);
            report.resize(REPORT_SIZE, 0);
            report
        };
        assert_eq!(
            reassembler.push(&report(br#"DSTART{"payload":{"success"#)),
            None
        );
        assert_eq!(
            reassembler.push(&report(br#"":1,"data":{"n":2}}}DEND"#)),
            Some(r#"{"payload":{"success":1,"data":{"n":2}}}"#.to_owned())
        );
    }

    #[test]
    fn stuffed_sentinel_inside_the_payload_survives() {
        let mut reassembler = Reassembler::default();
        let bytes = b"\x01DSTART{\"payload\":{\"m\":\"D\x00END\"}}DEND";
        assert_eq!(
            reassembler.push(bytes),
            Some(r#"{"payload":{"m":"DEND"}}"#.to_owned())
        );
    }

    #[test]
    fn a_stuffed_sentinel_split_across_reports_survives() {
        let mut reassembler = Reassembler::default();
        assert_eq!(
            reassembler.push(b"\x01DSTART{\"payload\":{\"name\":\"ADD"),
            None
        );
        assert_eq!(
            reassembler.push(b"\x01\x00END\"}}DEND\x00\x00"),
            Some(r#"{"payload":{"name":"ADDEND"}}"#.to_owned())
        );
        assert_eq!(
            reassembler.push(b"\x01DSTART{\"payload\":{\"m\":\"D\x00START\"}}DEND"),
            Some(r#"{"payload":{"m":"DSTART"}}"#.to_owned())
        );
    }

    #[test]
    fn junk_before_the_start_is_dropped_and_two_replies_are_separated() {
        let mut reassembler = Reassembler::default();
        assert_eq!(reassembler.push(b"\x01\x00\x00garbage"), None);
        assert_eq!(
            reassembler.push(b"\x01DSTART{\"a\":1}DEND\x01DSTART{\"b\":2}DEND"),
            Some(r#"{"a":1}"#.to_owned())
        );
        assert_eq!(reassembler.push(b""), Some(r#"{"b":2}"#.to_owned()));
        assert_eq!(reassembler.push(b""), None);
    }
}
