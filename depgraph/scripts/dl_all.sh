#!/usr/bin/env bash

# for f in ~/Projects/apictx/python-tools/each-proj/*.list ;
for f in ~/Projects/apictx/python-tools/each-proj--cli-usage-at-least-500/*.list ;
do
    _BASE_NAME=${f##*/}
    BASE_NAME=${_BASE_NAME%.list}
    echo "$BASE_NAME";
    RUST_LOG=info ../target/release/get_jar_remote -l "$f" \
        --dir "${HOME}"/.local/share/apictx-subjects/jars/limit-500/for-"${BASE_NAME}" \
        --storage ~/.local/share/apictx-subjects/jars/jars-storage/
done
