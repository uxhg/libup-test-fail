#!/usr/bin/env bash
set -eux

JAR_STORE="$HOME/data/jars-storage"

MVN_COORD=$1
FILE_NAME=$2
LINE_START=$3
LINE_END=$((LINE_START+${4:-51}))

DIRNAME=${MVN_COORD//:/--}
FOUND_FILE=$(find ${JAR_STORE}/${DIRNAME}/unpack   -iname ${FILE_NAME}.java -o -iname ${FILE_NAME}.groovy -o -iname  ${FILE_NAME}.scala )
echo $FOUND_FILE

if [[ -n $FOUND_FILE ]] ; then 
    bat --paging=always  -r ${LINE_START}:${LINE_END}  $FOUND_FILE;
fi
