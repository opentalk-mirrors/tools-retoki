# {{ product_name }} {{ version }}

- Release date: **{{ date }}**
- Release series: [**{{ series.version }} ({{ series.codename }})**](../README.md#{{ series.markdown_anchor }})

## Component versions

| Component | Version |
| --------- | ------- |
{% for component in components -%}
| **{{ component.identifier }}** | [{{ component.version }}]({{ component.gitlab_url }}/-/releases/v{{ component.version }}) |
{% endfor %}

---

Generation of this document was supported by [retoki](https://git.opentalk.dev/w.silbermayr/retoki).
