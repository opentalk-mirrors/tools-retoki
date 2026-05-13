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
    vcs_service::{Issue, Milestone, VcsService},
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
        let milestones = self.vcs_service.get_milestones()?;

        for milestone in milestones {
            self.update_milestone_issues(milestone)?;
        }

        Ok(())
    }

    fn update_milestone_issues(&mut self, milestone: Milestone) -> anyhow::Result<()> {
        let mut milestone_header = Some(
            format!("Milestone {}", milestone.title.green())
                .bold()
                .underline()
                .to_string(),
        );
        let issues = self
            .vcs_service
            .get_open_issues_with_milestone_and_label(&milestone.title, self.release_label)?;

        for issue in issues {
            self.update_issue(issue, &mut milestone_header)?;
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

        let before = description
            .lines()
            .take(start_marker_index.saturating_add(1));
        let after = description.lines().skip(end_marker_index);

        let tree = IssueWithBlockers::load_from_vcs_service(
            self.vcs_service,
            issue.clone(),
            MAX_DEPENDENCY_DEPTH,
        )?;

        let diagram_string = tree.to_mermaid_diagram(self.base_url);

        let overall_description = before.chain(diagram_string.lines()).chain(after).join("\n");

        if &overall_description == description {
            if self.dry_run {
                self.out.println(&format_args!(
                        "- {issue_ref}: No changes required in description. Running in DRY-RUN mode, issue description would remain unchanged:",
                    ));
                self.out.println(&format_args!("{}", overall_description));
            } else {
                self.out.println(&format_args!(
                    "- {issue_ref}: No changes required in description",
                ));
            }
            return Ok(());
        }

        if self.dry_run {
            self.out.println(&format_args!(
                "- {issue_ref}: Running in DRY-RUN mode, issue description would be updated to:",
            ));
            self.out.println(&format_args!("{}", overall_description));
            return Ok(());
        }

        self.vcs_service.update_issue_description(
            &issue.project.path_with_namespace,
            issue.iid,
            &overall_description,
        )?;
        self.out
            .println(&format_args!("- {issue_ref}: Description updated.",));
        Ok(())
    }
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
