## Component Release: {{ component_name }} {{ component_version }}

**Category:** {{ category }}
**Product Release:** {{ product_version }}
{% if previous_version %}**Previous Version:** {{ previous_version }}{% endif %}
{% if gitlab_url %}**Repository:** [{{ component_name }}]({{ gitlab_url }})
**Release:** [{{ component_version }}]({{ gitlab_url }}/-/releases/{{ component_version_prefixed }}){% endif %}

### Changes

<!-- Add component-specific changelog here -->

### Tasks

- [ ] Verify deployment
- [ ] Update documentation
- [ ] Notify stakeholders
