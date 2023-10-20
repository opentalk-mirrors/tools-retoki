# {{ component_name }} version history

{% for release in releases | reverse %}
---

# Version {{ release.version }}

{% if release.product_versions %}
(found in {{ product_name -}}{{- space -}}{%- for product_version in release.product_versions | reverse -%}
{%- if not loop.first -%},{{- space -}}{%- endif -%}
[**{{ product_version }}**](../releases/{{ product_version }}.md)
{%- endfor -%}
)
{%- endif %}

## Changelog

{{ release.changelog }}

{%- endfor -%}
