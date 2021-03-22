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
TOOL_BIN_PATH="../target/debug/"

RED='\033[0;31m'
NC='\033[0m' # no color

ERR_MVN_FAILED=5
ERR_CHDIR_FAILED=11

LOG_DIR="/tmp"
LOG_QL_DB_C="${LOG_DIR}/ql-db-c.log"

f_datadp() {
    cd "$MOD_PATH" || exit
    local QL_SCRIPT="$HOME/.local/tmp/codeql-repo/java/ql/src/datadp.ql"
    #local QL_SCRIPT="$HOME/.local/tmp/codeql-repo/java/ql/src/test.ql"
    DB_DIR_NAME="project.db"
    QL_CSV_NAME="project.csv"

    DB_DIR_PATH=${MOD_PATH:?}/${DB_DIR_NAME}
    if [ -d "$DB_DIR_PATH" ]; then
      echo "Project DB exists, deleting..."
      rm -rf "$DB_DIR_PATH"
      fi

    codeql database create "$DB_DIR_NAME" --language=java --command "mvn clean package -DskipTests" > $LOG_QL_DB_C
    if [ $? -ne 0 ]; then
      echo -e "${RED}CodeQL database create failed.${NC}"
      exit $ERR_MVN_FAILED
      fi

    codeql query run --database="$DB_DIR_NAME" --output=datadp.bqrs "$QL_SCRIPT"
    codeql bqrs decode --format=csv -o "$QL_CSV_NAME" datadp.bqrs
    cd "$OLDPWD" || exit $ERR_CHDIR_FAILED
}

f_cslicer() {
    cd "$MOD_PATH" || exit
    java -jar "$CSLICER_JAR" -e dl -ext dep -c "$CSLICER_CFG"
    cd "$OLDPWD" || exit $ERR_CHDIR_FAILED
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
${TOOL_BIN_PATH}/pomfact -i "$MOD_PATH" -o "${OUT_PATH}/PomDep.facts" --fmt souffle

# run codeql program for data flow
f_datadp

# generate facts from codeql results
${TOOL_BIN_PATH}/dpfact -i "${MOD_PATH}/${QL_CSV_NAME}" --ex "java."  --ex "<anonymous class>" -o "${OUT_PATH}/DataFlowVMethod.facts" --fmt souffle

# JAR-contain-Class facts and gen config for CSlicer
${TOOL_BIN_PATH}/clsfact -i "$MOD_PATH" --cslicer -o "${OUT_PATH}/ContainClass.facts"

# invoke CSlicer
f_cslicer

# if multiple module: need to move CSlicer generated facts into mod_path/.facts
# invoke souffle
#souffle-orig -F "$MOD_PATH/.facts"  '../dl/def.dl' -D '../dl/output'
