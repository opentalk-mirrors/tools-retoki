// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use indexmap::IndexMap;

use crate::data::{ComponentIdentifier, ComponentVersion};

pub(crate) fn display_option<T: ToString>(value: &Option<T>) -> String {
    value.as_ref().map(ToString::to_string).unwrap_or_default()
}

pub(crate) fn display_components(
    components: &IndexMap<ComponentIdentifier, ComponentVersion>,
) -> String {
    components
        .iter()
        .map(|(identifier, version)| format!("{identifier}: {version}"))
        .fold(String::new(), |a, b| {
            if a.is_empty() {
                b
            } else {
                a + ", " + b.as_str()
            }
        })
}
