pub use imp::*;

#[cfg(not(test))]
#[expect(
    clippy::cfg_not_test,
    reason = "tests run the same flows on a virtual clock and a local pool"
)]
mod imp {
    use std::{future::Future, time::Duration};

    #[must_use]
    pub fn now_millis() -> f64 {
        js_sys::Date::now()
    }

    pub async fn sleep(duration: Duration) {
        gloo_timers::future::sleep(duration).await;
    }

    pub fn spawn(task: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(task);
    }

    #[must_use]
    pub fn confirm(message: &str) -> bool {
        web_sys::window()
            .and_then(|window| window.confirm_with_message(message).ok())
            .unwrap_or(true)
    }

    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::float_arithmetic,
        reason = "a random number in 0..256 becomes one byte"
    )]
    pub fn random_bytes<const N: usize>() -> [u8; N] {
        let mut bytes = [0; N];
        let filled = web_sys::window()
            .and_then(|window| window.crypto().ok())
            .is_some_and(|crypto| crypto.get_random_values_with_u8_array(&mut bytes).is_ok());
        if filled {
            bytes
        } else {
            std::array::from_fn(|_| (js_sys::Math::random() * 256.0) as u8)
        }
    }
}

#[cfg(test)]
mod imp {
    use std::{
        cell::{Cell, RefCell},
        collections::VecDeque,
        future::Future,
        task::Poll,
        time::Duration,
    };

    use futures_executor::{LocalPool, LocalSpawner};
    use futures_util::task::LocalSpawnExt;

    thread_local! {
        static CLOCK: Cell<f64> = const { Cell::new(0.0) };
        static POOL: RefCell<LocalPool> = RefCell::new(LocalPool::new());
        static SPAWNER: LocalSpawner = POOL.with_borrow(LocalPool::spawner);
        static ANSWERS: RefCell<VecDeque<bool>> = const { RefCell::new(VecDeque::new()) };
        static QUESTIONS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
        static SEED: Cell<u8> = const { Cell::new(0) };
    }

    #[must_use]
    pub fn now_millis() -> f64 {
        CLOCK.get()
    }

    pub async fn sleep(duration: Duration) {
        CLOCK.set(duration.as_secs_f64().mul_add(1000.0, CLOCK.get()));
        let mut has_yielded = false;
        std::future::poll_fn(|context| {
            if has_yielded {
                Poll::Ready(())
            } else {
                has_yielded = true;
                context.waker().wake_by_ref();
                Poll::Pending
            }
        })
        .await;
    }

    pub fn spawn(task: impl Future<Output = ()> + 'static) {
        SPAWNER
            .with(|spawner| spawner.spawn_local(task))
            .expect("the test pool accepts tasks");
    }

    #[must_use]
    pub fn confirm(message: &str) -> bool {
        QUESTIONS.with_borrow_mut(|questions| questions.push(message.to_owned()));
        ANSWERS.with_borrow_mut(VecDeque::pop_front).unwrap_or(true)
    }

    #[must_use]
    pub fn random_bytes<const N: usize>() -> [u8; N] {
        let seed = SEED.get().wrapping_add(1);
        SEED.set(seed);
        [seed; N]
    }

    pub fn run_until_idle() {
        POOL.with_borrow_mut(LocalPool::run_until_stalled);
    }

    pub fn answer(answers: impl IntoIterator<Item = bool>) {
        ANSWERS.with_borrow_mut(|queue| queue.extend(answers));
    }

    #[must_use]
    pub fn questions() -> Vec<String> {
        QUESTIONS.with_borrow_mut(std::mem::take)
    }
}
