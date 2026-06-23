// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use indicatif::ProgressStyle;
use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt as _;

/// Configures and starts the progress bar associated with `span`.
///
/// When `length` is `Some`, a determinate progress bar with a position counter
/// is shown, otherwise a plain spinner is used. When `finish_message` is `Some`,
/// it is kept on screen once the span finishes, otherwise the bar is removed.
pub(crate) fn start(span: &Span, length: Option<u64>, message: &str, finish_message: Option<&str>) {
    let template = if length.is_some() {
        "{spinner:.green} {wide_bar} {pos}/{len} {msg}"
    } else {
        "{spinner:.green} {msg}"
    };
    span.pb_set_style(
        &ProgressStyle::with_template(template).expect("valid progress style template"),
    );
    if let Some(length) = length {
        span.pb_set_length(length);
    }
    span.pb_set_message(message);
    if let Some(finish_message) = finish_message {
        span.pb_set_finish_message(finish_message);
    }
    span.pb_start();
}
