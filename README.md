# Retoki

Generate release documentation for OpenTalk.

This tool will collect release notes from all components making up an OpenTalk product release.
A product release version consists of multiple component versions.

Creating a release is done by following these steps:

1. add the new product release to the `release.yaml`
   e.g. in the example `release.yaml` 2.8.0 would be a product release.

2. fetch changelogs
   `GITLAB_TOKEN=$(cat ~/.gitlab_token) retoki release 2.8.0 fetch-changelogs --profile internal`

   This will query the changelogs from the gitlab release entries of the components.

3. fetch product tickets
   `GITLAB_TOKEN=$(cat ~/.gitlab_token) retoki fetch-product-tickets --product-version 2.8.0`

   This will query the product changelog from all product work items with the `release-2.8.0` tag.

4. generate documentation
   `retoki generate --profile internal`

   Render the release documentation.

## Example `release.yaml`

```yaml
product_name: OpenTalk
releases_page_header: |
  ## Release Announcements
series:
  '24.8':
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
