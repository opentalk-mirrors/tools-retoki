{%- if show_md_header -%}
---
title: {{ product_name }} v{{ version }}
---

{% endif -%}
# {{ product_name }} v{{ version }}

{% if show_series_end_of_life -%}
Supported until: {{ end_of_life }}
{% endif %}
## Releases

{% for release in releases | reverse -%}
- [**v{{ release.version }}** ({{ release.date }})](../{{ release.version }}/README.md)
{% endfor %}
---

Generation of this document was supported by [retoki](https://git.opentalk.dev/opentalk/tools/retoki).
