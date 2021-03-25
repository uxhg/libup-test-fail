#!/usr/bin/env bash

RED='\033[0;31m'
NC='\033[0m' # no color

LOG_DIR="/tmp"
LOG_QL_DB_C="${LOG_DIR}/ql-db-c.log"

f_codeql() {
	local name_base=$(basename "$2")
	local script_name=${name_base%.ql}
	codeql database create "$1" --language=java --command "mvn clean package -DskipTests" > "$LOG_QL_DB_C"
	if [ $? -ne 0 ]; then
		echo -e "${RED}CodeQL database create failed.${NC}"
		exit 2
	fi
	codeql query run --database="$1" --output=${script_name}.bqrs "$2"
	codeql bqrs decode --format=csv -o ${script_name}.csv ${script_name}.bqrs
}


DB_DIR_NAME="$1"
QL_SCRIPT="$2"
f_codeql "$DB_DIR_NAME" "$QL_SCRIPT"
