#!/usr/bin/env bash
QL_SCRIPT=$HOME/.local/tmp/codeql-repo/java/ql/src/datadp.ql
# QL_SCRIPT=$HOME/.local/tmp/codeql-repo/java/ql/src/test.ql
DB_DIR_NAME="project.db"

codeql database create "$DB_DIR_NAME" --language=java --command "mvn clean package -DskipTests"
#codeql database analyze "$DB_DIR_NAME" "$QL_SCRIPT" --output=/tmp/a.csv --format=csv
#codeql bqrs decode --format=csv -o project.csv "$DB_DIR_NAME"/results/codeql-java/datadp.bqrs
codeql query run --database="$DB_DIR_NAME" --output=datadp.bqrs "$QL_SCRIPT"
codeql bqrs decode --format=csv -o project.csv datadp.bqrs

