#!/bin/bash
set -e # exit on first error
root="$(dirname "$0")"

endpoint=http://localhost:8000
tableName=issues
common_options="--endpoint-url $endpoint --no-cli-pager --no-cli-auto-prompt"

aws dynamodb delete-table $common_options --table-name $tableName > /dev/null
aws dynamodb create-table $common_options --table-name $tableName --cli-input-json file://$root/issues-table-schema.json
aws dynamodb list-tables $common_options
