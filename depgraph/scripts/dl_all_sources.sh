#!/usr/bin/env bash
set -eux

LIST_DIR=$1
LINK_BASE_DIR=$2
# link base dir in old experiments is  "${HOME}"/data/client-sources-symlink/

for f in ${LIST_DIR}/*.list;
# for f in ~/data/each-proj/*.list ;
# for f in ~/Projects/apictx/python-tools/each-proj--cli-usage-at-least-500/*.list ;
do
    _BASE_NAME=${f##*/}
    BASE_NAME=${_BASE_NAME%.list}
    echo "$BASE_NAME";
    RUST_LOG=info ../target/release/get_jar_remote -l "$f" \
        --dir "${LINK_BASE_DIR}"/for-"${BASE_NAME}" \
        --storage ~/data/jars-storage/ --sel "source" "test-source"
done
