#!/usr/bin/env bash
QL_SCRIPT=$HOME/.local/tmp/codeql-repo/java/ql/src/datadp.ql

codeql database create project.db --language=java
codeql database analyze project.db "$QL_SCRIPT" --output=/tmp/a.csv --format=csv
codeql bqrs decode --format=csv -o project.csv project.db/results/codeql-java/datadp.bqrs

