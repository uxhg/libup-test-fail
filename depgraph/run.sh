#!/usr/bin/env bash
# Requirements:
# + CodeQL command line tool (codeql executable on PATH)
# + Related CodeQL Program (set QL_SCRIPT)
# + CSlicer (set CSLICER_JAR)
# + Maven (mvn on PATH)

MOD_PATH="$1"
OUT_PATH="${MOD_PATH}/.facts"
CSLICER_JAR="$HOME/Projects/gitslice/target/cslicer-1.0.0-jar-with-dependencies.jar"
CSLICER_CFG="${MOD_PATH}/cslicer.properties"

function f_datadp() {
    cd "$MOD_PATH" || exit
    local QL_SCRIPT="$HOME/.local/tmp/codeql-repo/java/ql/src/datadp.ql"
    #local QL_SCRIPT="$HOME/.local/tmp/codeql-repo/java/ql/src/test.ql"
    DB_DIR_NAME="project.db"
    QL_CSV_NAME="project.csv"

    codeql database create "$DB_DIR_NAME" --language=java --command "mvn clean package -DskipTests"
    codeql query run --database="$DB_DIR_NAME" --output=datadp.bqrs "$QL_SCRIPT"
    codeql bqrs decode --format=csv -o "$QL_CSV_NAME" datadp.bqrs
    cd "$OLDPWD" || exit
}

function f_cslicer() {
    cd "$MOD_PATH" || exit
    java -jar "$CSLICER_JAR" -e dl -ext dep -c "$CSLICER_CFG"
    cd "$OLDPWD" || exit
}

if [ $# -lt 1 ]; then
    echo "Need a path to a maven module, exit."
    exit 2
fi

if [ ! -d "$OUT_PATH" ]; then
    mkdir "$OUT_PATH"
fi

set -x

# pom facts
target/debug/pomfact -i "$MOD_PATH" -o "${OUT_PATH}/PomDep.facts" --fmt souffle

# run codeql program for data flow
f_datadp

# generate facts from codeql results
target/debug/dpfact -i "${MOD_PATH}/${QL_CSV_NAME}" --ex "java."  --ex "<anonymous class>" -o "${OUT_PATH}/DataFlowVMethod.facts" --fmt souffle

# JAR-contain-Class facts and gen config for CSlicer
target/debug/clsfact -i "$MOD_PATH" --cslicer -o "${OUT_PATH}/ContainClass.facts"

# invoke CSlicer
f_cslicer


