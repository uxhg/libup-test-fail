#!/usr/bin/env bash

for f in ~/data/each-proj/*.list ;
# for f in ~/Projects/apictx/python-tools/each-proj--cli-usage-at-least-500/*.list ;
do
    _BASE_NAME=${f##*/}
    BASE_NAME=${_BASE_NAME%.list}
    echo "$BASE_NAME";
    RUST_LOG=info ../target/debug/get_jar_remote -l "$f" \
        --dir "${HOME}"/data/client-sources-link/for-"${BASE_NAME}" \
        --storage ~/data/jars-storage/ --sel "test" "test-source"
done
