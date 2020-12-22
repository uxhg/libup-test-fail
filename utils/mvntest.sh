#!/usr/bin/env bash
#OPTIND=1
#while getopts "ht:o:" opt; do
#    case "$opt" in 
#        h) 
#            show_help
#            exit 0
#            ;;
#        t) TestArg="-Dtest=$OPTARG"
#            ;;
#        o) LogFile=$OPTARG
#            ;;
#    esac
#if [[ -n $1 ]] ; then
#    echo "Run all tests"
#    TestArg="-Dtest=$1"
#fi
#LogFile=$2
#mvn -DtrimStackTrace=false test "$TestArg" > "$LogFile.log" 2>&1
#perl -pe 's/\e\[[0-9;]*m//g' "$IN" > "${LogFile%.log}-nocsi.log"
