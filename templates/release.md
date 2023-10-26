# {{ product_name }} {{ version }}

- Release date: **{{ date }}**
- Release series: [**{{ series.version }} ({{ series.codename }})**](../README.md#{{ series.markdown_anchor }})
{%- if previous %}
- Previous release: [**{{ previous }}**]({{ previous }}.md)
{%- endif %}
{%- if next %}
- Next release: [**{{ next }}**]({{ next }}.md)
{%- endif %}

{%- if release_notes %}

## Release notes

{{ release_notes -}}
{%- endif %}

## Component versions

| Component | Version |
| --------- | ------- |
{% for component in components -%}
| [**{{ component.identifier }}**](../components/{{ component.identifier }}.md) | [{{ component.version }}]({{ component.gitlab_url }}/-/releases/v{{ component.version }}) |
{% endfor %}

{% for component in components -%}
{% if component.identifier in component_releases -%}
---

{% for component_release in component_releases[component.identifier] | reverse -%}
## {{ component.identifier }} {{ component_release.version }}

{{ component_release.changelog }}
{% endfor -%}
{% endif -%}
{% endfor -%}

---

Generation of this document was supported by [retoki](https://git.opentalk.dev/opentalk/tools/retoki).
