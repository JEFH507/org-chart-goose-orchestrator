# API

## Config
- models.yaml: {name, provider, cost_per_1k_tokens, roles_allowed, sensitivity_level}

## Selection API (internal)
- select_models(task, policyHints) → {lead?:model, worker:model}
