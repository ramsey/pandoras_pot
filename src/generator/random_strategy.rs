use crate::config::GeneratorConfig;
use bytes::Bytes;
use rand::{
    distributions::{Alphanumeric, DistString},
    rngs::SmallRng,
    SeedableRng,
};
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::instrument;

use super::{GeneratorStrategy, P_TAG_SIZE};

/// Generates `chunk_size` of completely random text.
#[derive(Clone, Debug)]
pub(crate) struct Random {
    chunk_size: usize,
    sleep_delay: u64,
}

impl Random {
    pub fn new(chunk_size: usize, sleep_delay: u64) -> Self {
        Self {
            chunk_size,
            sleep_delay,
        }
    }
}

impl GeneratorStrategy for Random {
    #[instrument(name = "spawn_random", skip_all)]
    fn start(self, tx: mpsc::Sender<Bytes>) {
        let span = tracing::Span::current();
        tokio::task::spawn_blocking(move || {
            let _entered = span.enter();
            // No need to be secure, we are smacking bots
            let mut smol_rng = SmallRng::from_entropy();
            loop {
                let s = Alphanumeric.sample_string(&mut smol_rng, self.chunk_size - P_TAG_SIZE);
                let res = Bytes::from(format!("<p>\n{s}\n</p>\n"));

                if tx.blocking_send(res).is_err() {
                    break;
                }

                if self.sleep_delay > 0 {
                    sleep(Duration::from_millis(self.sleep_delay));
                }
            }
        });
    }
}

impl Default for Random {
    fn default() -> Self {
        Self::new(
            GeneratorConfig::default().chunk_size,
            GeneratorConfig::default().sleep_delay,
        )
    }
}
