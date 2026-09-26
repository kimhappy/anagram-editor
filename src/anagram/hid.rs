use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    time::Duration,
};

use futures_channel::oneshot;
use gloo_events::EventListener;
use gloo_timers::future::sleep;
use web_sys::HidDevice;

use crate::{
    devices::{DeviceError, hid, with_timeout},
    protocol::hid::{
        action::{
            Action, Notification, decode_reply, encode_notification, encode_request, reply_name,
        },
        envelope::{HidError, Reply},
        framing::{Reassembler, frame_request},
    },
};

const REPORT_GAP: Duration = Duration::from_millis(2);
const STALE_WINDOW_MILLIS: f64 = 3000.0;

struct Stale {
    name: String,
    until: f64,
}

#[derive(Default)]
struct Inbox {
    reassembler: Reassembler,
    pending: Option<(String, oneshot::Sender<String>)>,
    stale: Option<Stale>,
    is_closed: bool,
}

impl Inbox {
    fn receive(&mut self, report: &[u8], now: f64) {
        let mut unread = report;
        while let Some(document) = self.reassembler.push(unread) {
            unread = &[];
            self.deliver(document, now);
        }
    }

    fn deliver(&mut self, document: String, now: f64) {
        let name = reply_action(&document);
        if let Some(stale) = self.stale.take()
            && now < stale.until
            && name.as_deref() == Some(stale.name.as_str())
        {
            return;
        }
        let Some((expected, sender)) = self.pending.take() else {
            return;
        };
        if name.as_deref().is_none_or(|name| name == expected) {
            sender.send(document).unwrap_or_default();
        } else {
            self.pending = Some((expected, sender));
        }
    }

    fn close(&mut self) {
        self.is_closed = true;
        self.pending = None;
    }
}

fn reply_action(document: &str) -> Option<String> {
    serde_json::from_str::<Reply>(document).ok()?.action
}

#[derive(Default)]
struct Turnstile {
    busy: Cell<bool>,
    waiters: RefCell<VecDeque<oneshot::Sender<()>>>,
}

impl Turnstile {
    async fn acquire(self: &Rc<Self>) -> Pass {
        if !self.busy.replace(true) {
            return Pass(Rc::clone(self));
        }
        let (sender, receiver) = oneshot::channel();
        self.waiters.borrow_mut().push_back(sender);
        receiver.await.unwrap_or_default();
        Pass(Rc::clone(self))
    }

    fn release(&self) {
        let mut waiters = self.waiters.borrow_mut();
        let is_handed_over =
            std::iter::from_fn(|| waiters.pop_front()).any(|next| next.send(()).is_ok());
        if !is_handed_over {
            self.busy.set(false);
        }
    }
}

struct Pass(Rc<Turnstile>);

impl Drop for Pass {
    fn drop(&mut self) {
        self.0.release();
    }
}

pub struct HidClient {
    device: HidDevice,
    inbox: Rc<RefCell<Inbox>>,
    turnstile: Rc<Turnstile>,
    _listener: EventListener,
    _unplug_listener: Option<EventListener>,
}

impl Drop for HidClient {
    fn drop(&mut self) {
        drop(self.device.close());
    }
}

impl HidClient {
    pub async fn open(device: HidDevice) -> Result<Self, HidError> {
        hid::open(&device).await?;
        let inbox = Rc::new(RefCell::new(Inbox::default()));
        let listener = {
            let inbox = Rc::clone(&inbox);
            hid::on_input_report(&device, move |_, data| {
                inbox.borrow_mut().receive(&data, js_sys::Date::now());
            })
        };
        let unplug_listener = {
            let inbox = Rc::clone(&inbox);
            let own = device.clone();
            hid::on_disconnect(move |gone| {
                if gone == own {
                    inbox.borrow_mut().close();
                }
            })
            .ok()
        };
        Ok(Self {
            device,
            inbox,
            turnstile: Rc::new(Turnstile::default()),
            _listener: listener,
            _unplug_listener: unplug_listener,
        })
    }

    #[must_use]
    pub fn is_device(&self, device: &HidDevice) -> bool {
        *device == self.device
    }

    #[expect(clippy::float_arithmetic, reason = "a deadline in milliseconds")]
    pub async fn call<A: Action>(&self, action: A) -> Result<A::Data, HidError> {
        let _pass = self.turnstile.acquire().await;
        self.ensure_open()?;
        let json = encode_request(&action)?;
        let expected = reply_name::<A>();
        let deadline = A::TIMEOUT
            .as_secs_f64()
            .mul_add(1000.0, js_sys::Date::now());
        if A::VERIFIES_REPLY {
            self.inbox.borrow_mut().stale = None;
        }
        let mut receiver = self.expect_reply(&expected, true);
        self.send_frames(&json).await?;
        loop {
            let remaining =
                Duration::from_secs_f64((deadline - js_sys::Date::now()).max(0.0) / 1000.0);
            match with_timeout(receiver, remaining).await {
                Some(Ok(text)) => {
                    let data = decode_reply::<A>(&text)?;
                    if action.accepts(&data) {
                        return Ok(data);
                    }
                    receiver = self.expect_reply(&expected, false);
                }
                Some(Err(_cancelled)) => return Err(unplugged()),
                None => {
                    let mut inbox = self.inbox.borrow_mut();
                    inbox.pending = None;
                    inbox.stale = (!A::VERIFIES_REPLY).then(|| Stale {
                        name: expected,
                        until: js_sys::Date::now() + STALE_WINDOW_MILLIS,
                    });
                    return Err(HidError::Timeout(A::NAME));
                }
            }
        }
    }

    pub async fn notify<N: Notification>(&self, notification: N) -> Result<(), HidError> {
        let _pass = self.turnstile.acquire().await;
        self.ensure_open()?;
        let json = encode_notification(&notification)?;
        self.inbox.borrow_mut().reassembler.clear();
        self.send_frames(&json).await
    }

    fn ensure_open(&self) -> Result<(), HidError> {
        if self.inbox.borrow().is_closed {
            Err(unplugged())
        } else {
            Ok(())
        }
    }

    fn expect_reply(&self, expected: &str, is_fresh: bool) -> oneshot::Receiver<String> {
        let (sender, receiver) = oneshot::channel();
        let mut inbox = self.inbox.borrow_mut();
        if is_fresh {
            inbox.reassembler.clear();
        }
        inbox.pending = Some((expected.to_owned(), sender));
        receiver
    }

    async fn send_frames(&self, json: &str) -> Result<(), HidError> {
        for report in frame_request(json) {
            hid::send_report(&self.device, hid::anagram::REPORT_ID, &report).await?;
            sleep(REPORT_GAP).await;
        }
        Ok(())
    }
}

fn unplugged() -> HidError {
    HidError::Transport(DeviceError::Browser(
        "The Anagram was unplugged.".to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use futures_channel::oneshot;

    use super::{Inbox, STALE_WINDOW_MILLIS, Stale};

    const REPLY: &[u8] = b"\x01DSTART{\"action\":\"fetch_preset_res\",\"payload\":{}}DEND";

    fn waiting_inbox(stale_until: f64) -> (Inbox, oneshot::Receiver<String>) {
        let (sender, receiver) = oneshot::channel();
        let inbox = Inbox {
            pending: Some(("fetch_preset_res".to_owned(), sender)),
            stale: Some(Stale {
                name: "fetch_preset_res".to_owned(),
                until: stale_until,
            }),
            ..Inbox::default()
        };
        (inbox, receiver)
    }

    #[test]
    fn a_late_reply_inside_the_window_is_dropped_once() {
        let (mut inbox, mut receiver) = waiting_inbox(STALE_WINDOW_MILLIS);
        inbox.receive(REPLY, 0.0);
        assert_eq!(receiver.try_recv(), Ok(None));
        inbox.receive(REPLY, 0.0);
        assert!(receiver.try_recv().is_ok_and(|reply| reply.is_some()));
    }

    #[test]
    fn a_reply_sharing_a_report_with_a_stale_one_is_delivered() {
        let (mut inbox, mut receiver) = waiting_inbox(STALE_WINDOW_MILLIS);
        inbox.receive(&[REPLY, REPLY].concat(), 0.0);
        assert!(receiver.try_recv().is_ok_and(|reply| reply.is_some()));
    }

    #[test]
    fn a_reply_to_another_action_keeps_waiting() {
        let (mut inbox, mut receiver) = waiting_inbox(0.0);
        inbox.deliver(
            r#"{"action":"swap_presets_res","payload":{}}"#.to_owned(),
            0.0,
        );
        assert_eq!(receiver.try_recv(), Ok(None));
        assert!(inbox.pending.is_some());
        inbox.deliver(r#"{"payload":{}}"#.to_owned(), 0.0);
        assert!(receiver.try_recv().is_ok_and(|reply| reply.is_some()));
    }

    #[test]
    fn a_stale_mark_expires_so_retries_can_succeed() {
        let (mut inbox, mut receiver) = waiting_inbox(STALE_WINDOW_MILLIS);
        inbox.receive(REPLY, STALE_WINDOW_MILLIS);
        assert!(receiver.try_recv().is_ok_and(|reply| reply.is_some()));
    }
}
