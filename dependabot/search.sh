#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
	echo "This script needs a keyword for searching."
	exit
fi

for PP_NUM in {1..2}; do 
	curl \
		-H "Accept: application/vnd.github.v3+json" \
		"https://api.github.com/search/issues?q=$1+in:comments+is:pr+author:app/dependabot+language:java&sort=created&order=asc&sort=stars&per_page=100&page=$PP_NUM" \
		> "result${PP_NUM}.json"
done
