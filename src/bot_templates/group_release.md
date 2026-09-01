## Release: {{ group_name }} {{ group_version }}

**Product Release:** {{ product_version }}
{% if gitlab_url %}**Repository:** [{{ group_name }}]({{ gitlab_url }})
**Release:** [{{ group_version }}]({{ gitlab_url }}/-/releases/{{ group_version_prefixed }}){% endif %}

### Components

| Component | Category | Previous Version | New Version |
| --------- | -------- | ---------------- | ----------- |

{%- for row in components %}
| {{ row.component }} | {{ row.category }} | {% if row.old_version %}{{ row.old_version }}{% else %}—{% endif %} | {{ row.new_version }} |
{%- endfor %}

### Changes

<!-- Add component-specific changelog here -->

### Tasks

- [ ] Verify deployment
- [ ] Update documentation
- [ ] Notify stakeholders
