# {{ component_name }} version history

{% for release in releases | reverse %}
---

# {{ component_identifier }} v{{ release.version }}

{% if release.product_versions %}
(found in {{ product_name -}}{{- space -}}{%- for product_version in release.product_versions | reverse -%}
{%- if not loop.first -%},{{- space -}}{%- endif -%}
[**v{{ product_version }}**](../releases/{{ product_version }}.md)
{%- endfor -%}
)
{%- endif %}
{%- if release.changelog %}

## Changelog

{{ release.changelog }}
{%- endif %}

{%- endfor -%}
