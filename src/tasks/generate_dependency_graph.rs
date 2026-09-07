// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

mod issue_with_blockers;
mod update;

pub(crate) use update::{DependencyGraphUpdater, dependency_graph_for_issue};
