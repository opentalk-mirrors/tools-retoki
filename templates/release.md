# {{ product_name }} v{{ version }}

- Release date: **{{ date }}**
- Release series: [**{{ series.version }}{% if series.codename %} ({{ series.codename }}){% endif %}**](../README.md#{{ series.markdown_anchor }})
{%- if previous %}
- Previous release: [**v{{ previous }}**]({{ previous }}.md)
{%- endif %}
{%- if next %}
- Next release: [**v{{ next }}**]({{ next }}.md)
{%- endif %}

{%- if release_notes %}

## Release notes

{{ release_notes -}}
{%- endif %}

## Component versions

| Category | Component | Version |
| -------- | --------- | ------- |
{% for component in components -%}
| **{{ component.category }}** | [**{{ component.identifier }}**](../components/{{ component.identifier }}.md) | [v{{ component.version }}]({{ component.gitlab_url }}/-/releases/v{{ component.version }}) |
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
