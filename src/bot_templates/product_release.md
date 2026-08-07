## Component versions

<!-- markdownlint-disable MD056 -->
| Category | Component | Version | Ticket |
| -------- | --------- | ------- | ------ |
{% for category in categories -%}
{%- for component in category.components -%}
| **{{ category.name }}** | {% if component.gitlab_url %}[**{{ component.name }}**]({{ component.gitlab_url }}){% else %}**{{ component.name }}**{% endif %} | {% if component.gitlab_url %}[{{ component.prefixed_version }}]({{ component.gitlab_url }}/-/releases/{{ component.prefixed_version }}){% else %}{{ component.prefixed_version }}{% endif %}{% if component.has_changed %} :sparkles:{% endif %} | {% if component.ticket_url %}[Issue]({{ component.ticket_url }}){% elif component.has_changed %}TBD{% endif %} |
{% endfor -%}
{%- endfor %}
<!-- markdownlint-enable MD056 -->

## Dependency graph

<!-- DEPENDENCY_GRAPH_START -->
<!-- DEPENDENCY_GRAPH_END -->
