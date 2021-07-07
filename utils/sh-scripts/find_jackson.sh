#!/usr/bin/env sh
CUR_DIR_NAME=${PWD##*/} 
rg jackson .facts/20-deps/Call.facts | sort -u | tee "$CUR_DIR_NAME.jackson" | cut -f 2 |  sort -u  > "$CUR_DIR_NAME.jackson.api"
cp ./*.jackson.api ./*.jackson ./*.properties  "$HOME/Projects/lib-conflict/libup-test-fail/db.apiusage"
