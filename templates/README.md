{%- if show_md_header -%}
---
sidebar_position: 0
title: Release Bundles
---

{% endif -%}
# {{ product_name }} Release Bundles
{% if show_gantt_chart %}
```mermaid
---
displayMode: compact
---
gantt
    title       {{ product_name }} Releases
    dateFormat  YYYY-MM-DD
{% for serie in series %}
    section {{ serie.version }}{% if serie.codename %} {{ serie.codename }}{% endif %}
{%- for release in serie.releases %}
    {{ release.version }}  :{{ release.date }}, {{ release.end_date }}
{%- endfor %}
    EOL  :milestone, {{ serie.end_of_life }}
{% endfor %}
```
{% endif -%}
{% for serie in series | reverse %}
## {{ serie.version }}{% if serie.codename %} ({{ serie.codename }}){% endif %}
{% for release in serie.releases | reverse %}
- [**v{{ release.version }}** ({{ release.date }})](releases/{{ release.version }}.md)
{%- endfor %}
{% endfor %}

# Component lookup table

| Release | {% for component in components -%}{{ space }}[{{ component.identifier }}](components/{{ component.identifier }}.md){{ space }}|{%- endfor %}
| ------- |{% for component in components -%}{{ space }}----------{{ space }}|{%- endfor %}
{% for serie in series | reverse -%}
{%- for release in serie.releases | reverse -%}
| [**v{{ release.version }}**](releases/{{ release.version }}.md) |
{%- for component in components -%}
{%- set release_component = release.components_by_identifier | get(key=component.identifier, default=empty_release_component) -%}
{%- if 'version' in release_component -%}
{{- space }}{%- if show_gitlab_release_links and component.gitlab_url -%}[v{{ release_component.version }}]({{ component.gitlab_url }}/-/releases/v{{ release_component.version }}){%- else -%}v{{ release_component.version }}{%- endif -%}{{ space }}|
{%- else -%}
{{- space }}-{{ space }}|
{%- endif -%}
{%- endfor %}
{% endfor %}
{%- endfor %}

---

Generation of this document was supported by [retoki](https://git.opentalk.dev/opentalk/tools/retoki).
