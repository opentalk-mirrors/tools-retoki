# Retoki

Generate release documentation for OpenTalk.

This tool will collect release notes from all components making up an OpenTalk product release.
A product release version consists of multiple component versions.

Creating a release is done by following these steps:

1. add the new product release to the `release.yaml`
   e.g. in the example `release.yaml` 2.8.0 would be a product release.

2. fetch changelogs
   `GITLAB_TOKEN=$(cat ~/.gitlab_token) retoki release 2.8.0 fetch-changelogs`

   This will query the changelogs from the gitlab release entries of the components.

3. fetch product tickets
   `GITLAB_TOKEN="$(cat ~/.gitlab_token)" retoki release 2.8.0 fetch-product-tickets`

   This will query the product changelog from all product work items with the `release-2.8.0` tag.

4. generate documentation
   `retoki generate 2.8`

   Render the release documentation.

## Example `release.yaml`

```yaml
product_name: OpenTalk
releases_page_header: |
  ## Release Announcements
series:
  "24.8":
    end_of_life: 2024-09-19
    releases:
      24.8.0:
        date: 2024-07-19
        components:
          controller: 0.16.0
components:
  controller:
    name: OpenTalk Controller Community Edition
    category: services
    releases: {}
component_categories:
  services:
    name: Services
```

## Release ticket automation

In addition to generating documentation, `retoki` can manage the GitLab issues that track a
product release.

### Configuration

The automation commands (`ci` and `release <version> init`) talk to a GitLab instance. They are
configured, in order of increasing precedence, via a `retoki.toml` file, `RETOKI_` prefixed
environment variables, and the `GITLAB_TOKEN` environment variable:

| Setting              | `retoki.toml` key   | Environment variable       | Default                     |
| -------------------- | ------------------- | -------------------------- | --------------------------- |
| GitLab base URL      | `gitlab_url`        | `RETOKI_GITLAB_URL`        | `https://git.opentalk.dev`  |
| GitLab group         | `gitlab_group`      | `RETOKI_GITLAB_GROUP`      | `opentalk`                  |
| GitLab token         | `gitlab_token`      | `GITLAB_TOKEN`             | _(required)_                |
| Release ticket label | `release_label`     | `RETOKI_RELEASE_LABEL`     | `release-ticket`            |
| Release repository   | `release_repo`      | `RETOKI_RELEASE_REPO`      | `opentalk/product-releases` |
| `releases.yml` path  | `releases_yml_path` | `RETOKI_RELEASES_YML_PATH` | `./releases.yml`            |

### `retoki release <version> init`

Create or update the product release issue for a version in the release tickets project:

`GITLAB_TOKEN=$(cat ~/.gitlab_token) retoki release 2.8.0 init`

This seeds the release entry in `releases.yml` from the previous release (if necessary), builds
the component version table and creates or updates the corresponding GitLab issue.

### `retoki ci`

Run tasks intended to be executed in a CI pipeline. Currently this updates the dependency graphs
in all open release tickets:

`GITLAB_TOKEN=$(cat ~/.gitlab_token) retoki ci`

Both commands accept `--dry-run` (or `RETOKI_DRY_RUN=true`) to log the actions that would be
performed without writing anything to GitLab.

### `retoki release <version> announce`

Render a release announcement as Markdown, suitable for Matrix, Element and other chat channels:

`retoki release 2.8.0 announce`

To convert the announcement to plain text for email or mailing lists, pipe the output through a
converter such as [pandoc](https://pandoc.org/):

`retoki release 2.8.0 announce | pandoc -f markdown -t plain`
