// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use snafu::Whatever;
use url::Url;

use crate::{
    output::Output,
    vcs_service::{Issue, IssueLinkType, VcsService},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct IssueWithBlockers {
    pub issue: Issue,
    pub blocking_issues: BTreeSet<IssueWithBlockers>,
}

impl IssueWithBlockers {
    pub fn load_from_vcs_service(
        vcs_service: &dyn VcsService,
        issue: Issue,
        max_depth: usize,
    ) -> Result<IssueWithBlockers, Whatever> {
        if max_depth == 0 {
            return Ok(IssueWithBlockers {
                issue,
                blocking_issues: BTreeSet::new(),
            });
        }

        let remaining_depth = max_depth.saturating_sub(1);

        let blocking_issues = vcs_service
            .get_linked_issues(&issue.project.path_with_namespace, issue.iid)?
            .into_iter()
            .filter(|link| link.link_type == IssueLinkType::IsBlockedBy)
            .map(|link| Self::load_from_vcs_service(vcs_service, link.issue, remaining_depth))
            .collect::<Result<_, _>>()?;
        Ok(IssueWithBlockers {
            issue,
            blocking_issues,
        })
    }

    pub fn write_mermaid_diagram(&self, out: &mut dyn Output, base_url: &Url) {
        out.println(&format_args!("flowchart LR"));

        let issues_in_tree = self.issues_set();

        let _ = issues_in_tree.iter().fold(true, |is_first_output, issue| {
            if is_first_output {
                out.println(&format_args!(""));
            }

            out.println(&format_args!(
                "{}[<a href={}/-/issues/{} target=_blank>{}#{}</a><br>{}]",
                issue.mermaid_identifier(),
                base_url
                    .join(&issue.project.path_with_namespace)
                    .expect("base url must be joinable with project path"),
                issue.iid,
                issue.project.path_with_namespace,
                issue.iid,
                Self::escape_xml(&issue.title)
            ));
            false
        });

        self.write_dependencies(out, true);

        const OPEN_STYLE: &str = "";
        const CLOSED_STYLE: &str = "fill:#669";

        let _ = issues_in_tree
            .iter()
            .fold(true, |mut is_first_output, issue| {
                let style = if issue.state.is_opened() {
                    OPEN_STYLE
                } else {
                    CLOSED_STYLE
                };

                if !style.is_empty() {
                    if is_first_output {
                        out.println(&format_args!(""));
                        is_first_output = false;
                    }

                    out.println(&format_args!(
                        "style {} {}",
                        issue.mermaid_identifier(),
                        style
                    ));
                }
                is_first_output
            });
    }

    pub fn to_mermaid_diagram(&self, base_url: &Url) -> String {
        let mut diagram_buffer = Vec::new();
        diagram_buffer.println(&format_args!("```mermaid"));
        self.write_mermaid_diagram(&mut diagram_buffer, base_url);
        diagram_buffer.println(&format_args!("```"));

        String::from_utf8(diagram_buffer).expect("diagram export must be valid utf-8")
    }

    fn write_dependencies(&self, out: &mut dyn Output, prepend_newline: bool) {
        let _ = self
            .blocking_issues
            .iter()
            .fold(true, |is_first_output, blocker| {
                if prepend_newline && is_first_output {
                    out.println(&format_args!(""));
                }

                out.println(&format_args!(
                    "{} --> {}",
                    blocker.issue.mermaid_identifier(),
                    self.issue.mermaid_identifier(),
                ));
                blocker.write_dependencies(out, false);
                false
            });
    }

    fn issues_set(&self) -> BTreeSet<&Issue> {
        let mut set = BTreeSet::from_iter([&self.issue]);
        for blocking in &self.blocking_issues {
            set.extend(blocking.issues_set());
        }
        set
    }

    fn escape_xml(input: &str) -> String {
        input
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&apos;")
            .replace("(", "&#40;")
            .replace(")", "&#41;")
            .replace("[", "&#91;")
            .replace("]", "&#93;")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use pretty_assertions::assert_eq;
    use url::Url;

    use super::IssueWithBlockers;
    use crate::vcs_service::{Issue, IssueState, Project};

    #[test]
    fn single_issue() {
        let base_url: Url = "https://git.example.com".parse().expect("valid url");

        let project = Project {
            id: 123,
            path_with_namespace: "my/project".to_string(),
        };
        let issue = Issue {
            id: 150,
            iid: 93,
            title: "An issue <".to_string(),
            project,
            short_reference: "project#93".to_string(),
            description: Some("This is the issue description".to_string()),
            state: IssueState::Opened,
            linked_issues: vec![],
        };

        let tree = IssueWithBlockers {
            issue,
            blocking_issues: BTreeSet::new(),
        };

        assert_eq!(
            tree.to_mermaid_diagram(&base_url),
            "\
```mermaid
flowchart LR

my_project_93[<a href=https://git.example.com/my/project/-/issues/93 target=_blank>my/project#93</a><br>An issue &lt;]
```
"
        );
    }

    #[test]
    fn mixed_tree() {
        let base_url: Url = "https://git.example.com".parse().expect("valid url");

        let project_a = Project {
            id: 123,
            path_with_namespace: "my/project".to_string(),
        };
        let project_b = Project {
            id: 321,
            path_with_namespace: "another/project_b".to_string(),
        };

        let issue_a = Issue {
            id: 150,
            iid: 93,
            title: "issue a".to_string(),
            project: project_a.clone(),
            short_reference: "project_a#93".to_string(),
            description: Some("This is the issue description".to_string()),
            state: IssueState::Closed,
            linked_issues: vec![],
        };

        let issue_b = Issue {
            id: 12,
            iid: 55,
            title: "issue b (feature)".to_string(),
            project: project_b.clone(),
            short_reference: "project_b#55".to_string(),
            description: Some("This is the issue description".to_string()),
            state: IssueState::Closed,
            linked_issues: vec![],
        };

        let issue_c = Issue {
            id: 959,
            iid: 42,
            title: "[example] issue c".to_string(),
            project: project_b.clone(),
            short_reference: "project_b#42".to_string(),
            description: Some("This is the issue description".to_string()),
            state: IssueState::Closed,
            linked_issues: vec![],
        };

        let issue_d = Issue {
            id: 135,
            iid: 133,
            title: "issue d (bugfix)".to_string(),
            project: project_a.clone(),
            short_reference: "project_a#133".to_string(),
            description: Some("This is the issue description".to_string()),
            state: IssueState::Opened,
            linked_issues: vec![],
        };

        let tree = IssueWithBlockers {
            issue: issue_d,
            blocking_issues: BTreeSet::from([
                IssueWithBlockers {
                    issue: issue_b,
                    blocking_issues: BTreeSet::new(),
                },
                IssueWithBlockers {
                    issue: issue_c,
                    blocking_issues: BTreeSet::from([IssueWithBlockers {
                        issue: issue_a,
                        blocking_issues: BTreeSet::new(),
                    }]),
                },
            ]),
        };

        assert_eq!(
            tree.to_mermaid_diagram(&base_url),
            "\
```mermaid
flowchart LR

another_project_b_55[<a href=https://git.example.com/another/project_b/-/issues/55 target=_blank>another/project_b#55</a><br>issue b &#40;feature&#41;]
my_project_133[<a href=https://git.example.com/my/project/-/issues/133 target=_blank>my/project#133</a><br>issue d &#40;bugfix&#41;]
my_project_93[<a href=https://git.example.com/my/project/-/issues/93 target=_blank>my/project#93</a><br>issue a]
another_project_b_42[<a href=https://git.example.com/another/project_b/-/issues/42 target=_blank>another/project_b#42</a><br>&#91;example&#93; issue c]

another_project_b_55 --> my_project_133
another_project_b_42 --> my_project_133
my_project_93 --> another_project_b_42

style another_project_b_55 fill:#669
style my_project_93 fill:#669
style another_project_b_42 fill:#669
```
"
        );
    }
}
