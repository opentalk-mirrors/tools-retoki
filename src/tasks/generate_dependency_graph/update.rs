// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::bail;
use itertools::Itertools as _;
use owo_colors::OwoColorize as _;
use url::Url;

use crate::{
    output::Output,
    tasks::generate_dependency_graph::issue_with_blockers::IssueWithBlockers,
    vcs_service::{Issue, IssueFilter, IssueScope, IssueState, VcsService},
};

const START_MARKER: &str = "<!-- DEPENDENCY_GRAPH_START -->";
const END_MARKER: &str = "<!-- DEPENDENCY_GRAPH_END -->";
const MAX_DEPENDENCY_DEPTH: usize = 10usize;

pub(crate) struct DependencyGraphUpdater<'a> {
    vcs_service: &'a dyn VcsService,
    release_label: &'a str,
    out: &'a mut dyn Output,
    base_url: &'a Url,
    dry_run: bool,
}

impl<'a> DependencyGraphUpdater<'a> {
    pub fn new(
        vcs_service: &'a dyn VcsService,
        release_label: &'a str,
        out: &'a mut dyn Output,
        base_url: &'a Url,
        dry_run: bool,
    ) -> Self {
        Self {
            vcs_service,
            release_label,
            out,
            base_url,
            dry_run,
        }
    }

    pub fn apply(mut self) -> anyhow::Result<()> {
        let issues = self.vcs_service.fetch_issues(
            IssueScope::Group,
            &IssueFilter {
                labels: &[self.release_label],
                state: Some(IssueState::Opened),
            },
        )?;

        let mut section_header = Some("Release tickets".green().bold().underline().to_string());

        for issue in issues {
            self.update_issue(issue, &mut section_header)?;
        }

        Ok(())
    }

    fn print_section_header(&mut self, section_header: &str) {
        self.out.println(&format_args!(""));
        self.out.println(&format_args!("{section_header}"));
        self.out.println(&format_args!(""));
    }

    fn update_issue(
        &mut self,
        issue: Issue,
        section_header: &mut Option<String>,
    ) -> anyhow::Result<()> {
        let issue_ref = format!("{}#{}", issue.project.path_with_namespace, issue.iid)
            .blue()
            .to_string();

        let Some(description) = issue.description.as_ref() else {
            return Ok(());
        };

        let (start_marker_index, end_marker_index) = match get_dependency_graph_markers(description)
        {
            Ok(Some(marker_indices)) => marker_indices,
            Ok(None) => return Ok(()),
            Err(e) => {
                if let Some(section_header) = section_header.take() {
                    self.print_section_header(&section_header);
                }
                self.out
                    .println(&format_args!("- {issue_ref}: {}", e.red()));
                return Ok(());
            }
        };

        if let Some(section_header) = section_header.take() {
            self.print_section_header(&section_header);
        }

        let overall_description = build_description_with_graph(
            self.vcs_service,
            &issue,
            description,
            self.base_url,
            start_marker_index,
            end_marker_index,
        )?;

        if &overall_description == description {
            self.out.println(&format_args!(
                "- {issue_ref}: No changes required in description",
            ));
            return Ok(());
        }

        if self.dry_run {
            self.out.println(&format_args!(
                "- {issue_ref}: Running in DRY-RUN mode, issue description would be updated to:",
            ));
            self.out.println(&format_args!("{}", overall_description));
        }

        self.vcs_service.update_issue_description(
            &issue.project.path_with_namespace,
            issue.iid,
            &overall_description,
        )?;
        Ok(())
    }
}

/// Render the mermaid dependency graph for `issue` and its blockers.
///
/// The returned string is the fenced ```mermaid block, without a trailing
/// newline, ready to be embedded into an issue template.
pub(crate) fn dependency_graph_for_issue(
    vcs_service: &dyn VcsService,
    issue: &Issue,
    base_url: &Url,
) -> anyhow::Result<String> {
    let tree =
        IssueWithBlockers::load_from_vcs_service(vcs_service, issue.clone(), MAX_DEPENDENCY_DEPTH)?;
    Ok(tree.to_mermaid_diagram(base_url).trim_end().to_owned())
}

/// Replace the content between the dependency graph markers of `description`
/// with the mermaid diagram rendered from `issue` and its blockers.
fn build_description_with_graph(
    vcs_service: &dyn VcsService,
    issue: &Issue,
    description: &str,
    base_url: &Url,
    start_marker_index: usize,
    end_marker_index: usize,
) -> anyhow::Result<String> {
    let before = description
        .lines()
        .take(start_marker_index.saturating_add(1));
    let after = description.lines().skip(end_marker_index);

    let diagram_string = dependency_graph_for_issue(vcs_service, issue, base_url)?;

    Ok(before.chain(diagram_string.lines()).chain(after).join("\n"))
}

fn get_dependency_graph_markers(s: &str) -> anyhow::Result<Option<(usize, usize)>> {
    let enumerated_lines = s.lines().enumerate();

    let mut start_markers = enumerated_lines.clone().filter(|(_, l)| *l == START_MARKER);
    let mut end_markers = enumerated_lines.clone().filter(|(_, l)| *l == END_MARKER);

    let start_marker = start_markers.next();
    if start_markers.next().is_some() {
        bail!("Found multiple start markers, aborting");
    }

    let end_marker = end_markers.next();
    if end_markers.next().is_some() {
        bail!("Found multiple end markers, aborting");
    }

    match (start_marker, end_marker) {
        (Some((start_line, _)), Some((end_line, _))) if start_line > end_line => {
            bail!("Found end line before start line, aborting");
        }
        (Some((start_line, _)), Some((end_line, _))) => Ok(Some((start_line, end_line))),
        (None, None) => Ok(None),
        _ => {
            bail!("Inconsistent start and end markers, aborting");
        }
    }
}
