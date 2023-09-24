# {{ product_name }} Release Bundles
{% for serie in series %}
## {{ serie.version }} ({{ serie.codename }})
{% for release in serie.releases %}
- [{{ release.version }} ({{ release.date }})](releases/{{ release.version }}.md)
{%- endfor %}
{% endfor %}
