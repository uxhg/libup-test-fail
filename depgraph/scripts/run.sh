#!/usr/bin/env bash
# Requirements:
# + CodeQL command line tool (codeql executable on PATH)
# + Related CodeQL Program (set QL_SCRIPT)
# + CSlicer (set CSLICER_JAR)
# + Maven (mvn on PATH)

MOD_PATH=$1
#PROJ_NAME=$(basename "$MOD_PATH")
OUT_PATH=${MOD_PATH}/.facts
CSLICER_JAR=$HOME/Projects/gitslice/target/cslicer-1.0.0-jar-with-dependencies.jar
CSLICER_CFG=${MOD_PATH}/cslicer.properties
TOOL_BIN_PATH=../target/release/
THIS_TOOL_DIR=$HOME/Projects/lib-conflict/libup-test-fail/
DL_PROGRAM_DIR=${THIS_TOOL_DIR}/depgraph/datalog
DL_OUT_DIR=${THIS_TOOL_DIR}/dl/output

export PATH="$TOOL_BIN_PATH:$PATH"

RED='\033[0;31m'
NC='\033[0m' # no color

ERR_MVN_FAILED=5
ERR_CHDIR_FAILED=11

LOG_DIR=/tmp
LOG_QL_DB_C=${LOG_DIR}/ql-db-c.log

QL_SCRIPT=$HOME/.local/tmp/codeql-repo/java/ql/src/datadp.ql
#QL_SCRIPT="$HOME/.local/tmp/codeql-repo/java/ql/src/test.ql"
_QL_NAME=$(basename "$QL_SCRIPT")
QL_NAME=${_QL_NAME%.ql}
QL_RESULT_CSV=${QL_NAME}.csv


f_get_mvn_coord_id() {
	# $1 module path
	# $2 groupId, artifactId, etc.
	cd "$1" || exit $ERR_CHDIR_FAILED
	local target_id
	target_id=$(mvn org.apache.maven.plugins:maven-help-plugin:3.2.0:evaluate -q -DforceStdout -Dexpression=project."${2}")
	cd "$OLDPWD" || exit $ERR_CHDIR_FAILED
	echo -n "$target_id"
}

f_datadp() {
	cd "$MOD_PATH" || exit
	local DB_DIR_NAME="project.db"
	# local QL_CSV_NAME="project.csv"

	DB_DIR_PATH=${MOD_PATH:?}/${DB_DIR_NAME}
	if [ -d "$DB_DIR_PATH" ]; then
		echo "Project DB ($DB_DIR_PATH) exists, deleting..."
		rm -rf "$DB_DIR_PATH"
	fi
	f_codeql "$DB_DIR_NAME" "$QL_SCRIPT"
	cd "$OLDPWD" || exit $ERR_CHDIR_FAILED
}

f_codeql() {
	# $1: directory name of codeql generated database
	# $2: path to the codeql script
	codeql database create "$1" --language=java --command \
		   "mvn clean package -DskipTests" > $LOG_QL_DB_C
	if [ $? -ne 0 ]; then
		echo -e "${RED}CodeQL database create failed.${NC}"
		if [ -d "${MOD_PATH:?}/target" ]; then
			echo -e "${RED}Consider deleting generated class files in target/ ${NC}"
		fi
		exit $ERR_MVN_FAILED
	fi
	codeql query run --database="$1" --output="${QL_NAME}.bqrs" "$2"
	codeql bqrs decode --format=csv -o "${QL_NAME}.csv" "${QL_NAME}.bqrs"
}

f_cslicer() {
	cd "$MOD_PATH" || exit $ERR_CHDIR_FAILED
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

GRP_ID=$(f_get_mvn_coord_id "$MOD_PATH" "groupId")
ART_ID=$(f_get_mvn_coord_id "$MOD_PATH" "artifactId")
PROJ_NAME=${GRP_ID}--${ART_ID}

#cd $MOD_PATH && mvn clean package -DskipTests && cd $OLDPWD
# pom facts
pomfact -i "$MOD_PATH" \
				-o "${OUT_PATH}/PomDep.facts" \
				--fmt souffle

# run codeql program for data flow
# f_datadp

# generate facts from codeql results
# dpfact -i "${MOD_PATH}/${QL_RESULT_CSV}"\
# 				--ex "java."  --ex "<anonymous class>" \
# 				-o "${OUT_PATH}/DataFlowVMethod.facts" \
# 				--fmt souffle

# JAR-contain-Class facts and gen config for CSlicer
clsfact -i "$MOD_PATH" --cslicer \
				-o "${OUT_PATH}/ContainClass.facts"

# invoke CSlicer
f_cslicer

# if multiple module: need to move CSlicer generated facts into mod_path/.facts
if [ ! -d "$DL_OUT_DIR" ]; then
	mkdir -p "$DL_OUT_DIR"
fi

# invoke souffle
souffle-orig -F "$MOD_PATH/.facts"  "${DL_PROGRAM_DIR}/simple-dataflow.dl" -D "${DL_OUT_DIR}/${PROJ_NAME}"
