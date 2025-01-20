use indicatif::{ProgressBar, ProgressStyle};
use std::future::Future;

pub struct Progress {
    progress_bar: ProgressBar,
}

impl Progress {
    pub fn new() -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );
        Self { progress_bar: pb }
    }

    pub async fn run<F, T>(&self, message: &str, future: F) -> T
    where
        F: Future<Output = T>,
    {
        self.progress_bar.set_message(message.to_string());
        let result = future.await;
        self.progress_bar.finish_with_message(message.to_string());
        result
    }
}
