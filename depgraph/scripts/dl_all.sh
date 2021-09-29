#!/usr/bin/env bash

for f in ~/Projects/apictx/python-tools/each-proj/*.list ;
do
    _BASE_NAME=${f##*/}
    BASE_NAME=${_BASE_NAME%.list}
    echo "$BASE_NAME";
    RUST_LOG=info ../target/debug/get_jar_remote -l "$f" \
        --dir "${HOME}"/.local/share/apictx-subjects/jars/for-"${BASE_NAME}" \
        --storage ~/.local/share/apictx-subjects/jars/jars-storage/
done
