// NOTE: Don't know about the name of this crate.
// Maybe malbox-fetcher? Open to suggestions.

mod error;

pub use error::Error;
use error::Result;
use indicatif::{ProgressBar, ProgressStyle};

use tokio::{
    fs::{remove_file, File},
    io::{self, AsyncReadExt, AsyncWriteExt},
    join,
};

use tokio_stream::{self as stream, StreamExt};

// FIXME:
// - Fix length/duration for progress bar
// - Change styling for progress bar
// - Refactor code
pub async fn download_files(url: &str, path: &str) -> Result<()> {
    let mut file = File::create(path).await?;
    let stream = reqwest::get(url).await?;

    let bar = ProgressBar::new(stream.content_length().unwrap());

    bar.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {percent}% : {msg}")
            .unwrap(),
    );

    let mut stream = stream.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
        bar.inc(1)
    }

    file.flush().await?;

    bar.finish();

    Ok(())
}
