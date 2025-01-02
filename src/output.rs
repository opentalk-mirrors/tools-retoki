// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Arguments, io::Write};

pub(crate) trait Output {
    fn println(&mut self, value: &Arguments);
}

impl<W: Write> Output for W {
    fn println(&mut self, value: &Arguments) {
        let _ = writeln!(self, "{}", value);
    }
}

#[cfg(test)]
mod dummy_output;

#[cfg(test)]
pub(crate) use dummy_output::DummyOutput;
