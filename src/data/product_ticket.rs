// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductTicket {
    title: String,
    iid: u64,
    web_url: String,
    labels: Vec<String>,
}

impl ProductTicket {
    pub fn new(title: String, iid: u64, web_url: String, labels: Vec<String>) -> Self {
        Self {
            title,
            iid,
            web_url,
            labels,
        }
    }
}
