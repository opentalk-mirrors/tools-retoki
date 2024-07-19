{%- if show_md_header -%}
---
sidebar_position: {{ sidebar_position }}
title: {{ product_name }} v{{ version }}
---

{% endif -%}
# {{ product_name }} v{{ version }}

- Release date: **{{ date }}**
- Release series: [**{{ series.version }}{% if series.codename %} ({{ series.codename }}){% endif %}**](../README.md#{{ series.markdown_anchor }})
{%- if previous %}
- Previous release: [**v{{ previous }}**](../{{ previous }}/README.md)
{%- endif %}
{%- if next %}
- Next release: [**v{{ next }}**](../{{ next }}/README.md)
{%- endif %}

{%- if release_notes %}

## Release notes

{{ release_notes -}}
{%- endif %}

## Component versions

| Category | Component | Version |
| -------- | --------- | ------- |
{% for component in components -%}
| **{{ component.category }}** | [**{{ component.identifier }}**](../components/{{ component.identifier }}.md) | {%- if show_gitlab_release_links and component.gitlab_url -%}[v{{ component.version }}]({{ component.gitlab_url }}/-/releases/v{{ component.version }}){%- else -%}v{{ component.version }}{%- endif -%} |
{% endfor -%}

{% for component in components %}
{%- if component.identifier in component_releases %}
---
{% for component_release in component_releases[component.identifier] | reverse %}
## {{ component.identifier }} v{{ component_release.version }}
{% if component_release.changelog %}
{{ component_release.changelog }}
{%- endif -%}
{% endfor -%}
{% endif -%}
{% endfor %}

---

Generation of this document was supported by [retoki](https://git.opentalk.dev/opentalk/tools/retoki).
