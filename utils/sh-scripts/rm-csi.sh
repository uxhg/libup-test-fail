#!/usr/bin/env bash
IN="$1"
perl -pe 's/\e\[[0-9;]*m//g' "$IN" > "${IN%.log}-nocsi.log"
