We are happy to announce a new {{ product_name }} release: **v{{ version }}**.

{%- if release_notes %}

## Release notes

{{ release_notes -}}
{%- endif %}

## Component versions

| Category | Component | Version |
| -------- | --------- | ------- |
{%- for component in components %}
| **{{ component.category }}** | {{ component.identifier }} | {% if show_gitlab_release_links and component.gitlab_url -%}[{{ component.prefixed_version }}]({{ component.gitlab_url }}/-/releases/{{ component.prefixed_version }}){%- else -%}{{ component.prefixed_version }}{%- endif %} |
{%- endfor %}
