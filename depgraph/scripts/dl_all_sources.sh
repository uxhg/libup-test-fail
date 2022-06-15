#!/usr/bin/env bash
# the paths are configured to be used on server csl01
# and needed to be re-configured if used on other machines

for f in ~/data/each-proj/*.list ;
# for f in ~/Projects/apictx/python-tools/each-proj--cli-usage-at-least-500/*.list ;
do
    _BASE_NAME=${f##*/}
    BASE_NAME=${_BASE_NAME%.list}
    echo "$BASE_NAME";
    RUST_LOG=info ../target/release/get_jar_remote -l "$f" \
        --dir "${HOME}"/data/client-sources-symlink/for-"${BASE_NAME}" \
        --storage ~/data/jars-storage/ --sel "source" "test-source"
done
