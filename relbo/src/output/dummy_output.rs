// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use derive_more::derive::{AsRef, Display, Into};

use super::Output;

#[derive(Default, Display, AsRef, Into)]
pub(crate) struct DummyOutput(String);

impl DummyOutput {
    pub fn new() -> Self {
        Self(String::new())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Output for DummyOutput {
    fn println(&mut self, value: &std::fmt::Arguments) {
        self.0.push_str(&format!("{}\n", value));
    }
}
