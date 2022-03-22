#!/usr/bin/env bash

JAR_STORE="$HOME/data/jars-storage"

MVN_COORD=$1
FILE_NAME=$2
LINE_NUM=$3
LINE_MAX=$4

DIRNAME=${MVN_COORD//:/--}
FOUND_FILE=$(find ${JAR_STORE}/${DIRNAME}/unpack -iname ${FILE_NAME}.java)
echo $FOUND_FILE

if [[ -n $FOUND_FILE ]] ; then 
bat -r ${LINE_NUM}:$((LINE_NUM+${LINE_MAX:-20}))  $FOUND_FILE;
fi
