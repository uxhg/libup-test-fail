#!/usr/bin/env bash
# a simple script to remove 404 "jar"s wrongly downloaded before fixing the download tool
# used together with find . -type f -iname "*.jar" -exec /path/to/rm404.sh {} \;
# need to make sure the relative paths (i.e., the string in $1) do not contain semicolon
# otherwise, need to modify the cut command
set -eu

if [[ $(file "$1" | cut -d: -f2) ==  " HTML document, ASCII text" ]]; then 
    # echo will remove "$1";
    rm "$1";
else
    echo "not 404";
fi
