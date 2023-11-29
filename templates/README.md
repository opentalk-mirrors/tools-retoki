# {{ product_name }} Release Bundles

```mermaid
---
displayMode: compact
---
gantt
    title       {{ product_name }} Releases
    dateFormat  YYYY-MM-DD
{% for serie in series %}
    section {{ serie.version }} {{ serie.codename }}
{%- for release in serie.releases %}
    {{ release.version }}  :{{ release.date }}, {{ release.end_date }}
{%- endfor %}
    EOL  :milestone, {{ serie.end_of_life }}
{% endfor %}
```

{% for serie in series | reverse %}
## {{ serie.version }} ({{ serie.codename }})
{% for release in serie.releases | reverse %}
- [**v{{ release.version }}** ({{ release.date }})](releases/{{ release.version }}.md)
{%- endfor %}
{% endfor %}

# Component lookup table

| Release | {% for component in components -%}{{ space }}[{{ component.identifier }}](components/{{ component.identifier }}.md){{ space }} | {%- endfor %} |
| ------- |{% for component in components -%}{{ space }}----------{{ space }}|{%- endfor %}
{% for serie in series | reverse -%}
{%- for release in serie.releases | reverse -%}
| [**v{{ release.version }}**](releases/{{ release.version }}.md) |
{%- for component in components -%}
{%- set release_component = release.components_by_identifier | get(key=component.identifier, default=empty_release_component) -%}
{%- if 'version' in release_component -%}
{{- space }}[v{{ release_component.version }}]({{ component.gitlab_url }}/-/releases/v{{ release_component.version }}){{ space }}|
{%- else -%}
{{- space }}-{{ space }}|
{%- endif -%}
{%- endfor %}
{% endfor %}
{%- endfor %}

---

Generation of this document was supported by [retoki](https://git.opentalk.dev/opentalk/tools/retoki).
