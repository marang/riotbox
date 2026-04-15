use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread::{self, JoinHandle},
    time::Duration,
};

use riotbox_core::TimestampMs;

pub const DEFAULT_RUNTIME_PULSE_INTERVAL: Duration = Duration::from_millis(20);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSignal {
    TransportPulse { timestamp_ms: TimestampMs },
}

pub struct RuntimePulseSource {
    receiver: Receiver<RuntimeSignal>,
    stop_sender: Option<mpsc::Sender<()>>,
    join_handle: Option<JoinHandle<()>>,
}

impl RuntimePulseSource {
    #[must_use]
    pub fn spawn(interval: Duration) -> Self {
        let (signal_sender, receiver) = mpsc::channel();
        let (stop_sender, stop_receiver) = mpsc::channel();

        let join_handle = thread::Builder::new()
            .name("riotbox-runtime-pulse".into())
            .spawn(move || {
                loop {
                    let signal = RuntimeSignal::TransportPulse {
                        timestamp_ms: timestamp_now_ms(),
                    };
                    if signal_sender.send(signal).is_err() {
                        break;
                    }

                    match stop_receiver.recv_timeout(interval) {
                        Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                    }
                }
            })
            .expect("spawn runtime pulse thread");

        Self {
            receiver,
            stop_sender: Some(stop_sender),
            join_handle: Some(join_handle),
        }
    }

    pub fn drain_latest(&self) -> Option<RuntimeSignal> {
        let mut latest = None;

        loop {
            match self.receiver.try_recv() {
                Ok(signal) => latest = Some(signal),
                Err(TryRecvError::Empty) => return latest,
                Err(TryRecvError::Disconnected) => return latest,
            }
        }
    }
}

impl Drop for RuntimePulseSource {
    fn drop(&mut self) {
        if let Some(stop_sender) = self.stop_sender.take() {
            let _ = stop_sender.send(());
        }

        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

fn timestamp_now_ms() -> TimestampMs {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as TimestampMs
}
