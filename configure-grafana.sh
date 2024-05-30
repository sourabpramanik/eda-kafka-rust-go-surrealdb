#!/bin/bash

GRAFANA_PROMTAIL_CONFIG_FILE="promtail-config.yml"

_escapeRegex() {
    printf '%s\n' "$1" | sed 's/[\[\.*^$/]/\\&/g'
}

_modifyYml() {
    local MOD_FILE="$1"
    local MOD_STR="$(_escapeRegex "$2")"
    local NEW_VAL="$(_escapeRegex "$3")"
    sed -i.bak "s/$MOD_STR/$NEW_VAL/g" "$MOD_FILE"
}

cp promtail-config-blank.yml "$GRAFANA_PROMTAIL_CONFIG_FILE"
_modifyYml "$GRAFANA_PROMTAIL_CONFIG_FILE" "\${GRAFANA_LOGS_WRITE_URL}" "$GRAFANA_LOGS_WRITE_URL"
_modifyYml "$GRAFANA_PROMTAIL_CONFIG_FILE" "\${GRAFANA_LOGS_USERNAME}" "$GRAFANA_LOGS_USERNAME"
_modifyYml "$GRAFANA_PROMTAIL_CONFIG_FILE" "\${GRAFANA_LOGS_API_KEY}" "$GRAFANA_LOGS_API_KEY"
echo "Created: $GRAFANA_PROMTAIL_CONFIG_FILE"
