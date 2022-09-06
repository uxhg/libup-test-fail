#!/usr/bin/env bash

LIST_DIR=$1
LINK_BASE_DIR=$2
# in old experiments, this dir is  "${HOME}"/data/limit-100/ on server
#   or ~/.local/share/apictx-subjects/jars/limit-500 on local

for f in "$LIST_DIR"/*.list;
# for f in ~/data/each-proj/*.list ; # server
# for f in ~/Projects/apictx/python-tools/each-proj--cli-usage-at-least-500/*.list ; # local 
do
    _BASE_NAME=${f##*/}
    BASE_NAME=${_BASE_NAME%.list}
    echo "$BASE_NAME";
    RUST_LOG=info ../target/release/get_jar_remote -l "$f" \
        --dir "${LINK_BASE_DIR}"/for-"${BASE_NAME}" \
        --storage ~/data/jars-storage/
        # --storage /home1/xh/data/csl01-data/jars-storage  # storage path on cslu2
        # --storage ~/.local/share/apictx-subjects/jars/jars-storage/  # storage path on local machine
done
