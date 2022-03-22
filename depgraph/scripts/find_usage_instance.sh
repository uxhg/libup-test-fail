#!/usr/bin/env bash

JAR_STORE="$HOME/data/jars-storage"

MVN_COORD=$1
FILE_NAME=$2
LINE_NUM=$3

DIRNAME=${MVN_COORD//:/--}
bat -r ${LINE_NUM}:$((LINE_NUM+20)) $(find ${JAR_STORE}/${DIRNAME}/unpack -iname ${FILE_NAME}.java)

