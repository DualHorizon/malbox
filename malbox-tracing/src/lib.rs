use ansi_term::Colour::{Blue, Cyan, Green, Red, Yellow};
use ansi_term::Style;
use std::fmt;
use tracing_subscriber::{
    fmt::{
        format::Writer,
        time::{FormatTime, SystemTime},
        FormatEvent, FormatFields, Layer,
    },
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter,
};

struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> fmt::Result {
        let timer = SystemTime::default();
        timer.format_time(&mut writer)?;
        write!(writer, " ")?;

        let level = match *event.metadata().level() {
            tracing::Level::ERROR => Red.bold().paint("ERROR"),
            tracing::Level::WARN => Yellow.bold().paint("WARN"),
            tracing::Level::INFO => Green.bold().paint("INFO"),
            tracing::Level::DEBUG => Blue.bold().paint("DEBUG"),
            tracing::Level::TRACE => Style::new().dimmed().paint("TRACE"),
        };
        write!(writer, "{} ", level)?;

        write!(writer, "{} ", Cyan.paint(event.metadata().target()))?;

        if let (Some(file), Some(line)) = (event.metadata().file(), event.metadata().line()) {
            write!(writer, "{} ", Yellow.paint(format!("{}:{}", file, line)))?;
        }

        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

pub fn init_tracing(log_level: &str) {
    let fmt_layer = Layer::default()
        .event_format(CustomFormatter)
        .with_ansi(true);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("malbox={}", log_level)));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
