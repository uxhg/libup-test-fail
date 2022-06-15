#!/usr/bin/env bash

# change this to correct path where jars are stored
JAR_STORE="$HOME/data/jars-storage"

MVN_COORD=$1
FILE_NAME=$2
LINE_NUM=$3
LINE_MAX=$4

DIRNAME=${MVN_COORD//:/--}
FOUND_FILE=$(find ${JAR_STORE}/${DIRNAME}/unpack   -iname ${FILE_NAME}.java -o -iname ${FILE_NAME}.groovy -o -iname  ${FILE_NAME}.scala )
echo $FOUND_FILE

if [[ -n $FOUND_FILE ]] ; then 
    bat --paging=always  -r ${LINE_NUM}:$((LINE_NUM+${LINE_MAX:-51}))  $FOUND_FILE;
fi
