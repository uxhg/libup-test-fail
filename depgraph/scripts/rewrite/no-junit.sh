#!/usr/bin/env sh

MODPATH=$1

# find all files containing @Before
cd $MODPATH
rg -l '\s+@Before' -t java | xargs -I '{}'  sed -i -r -e 's/^(\s+)(@Before)/\1\/\/\2/g' {}
# some projects check formatting
mvn com.coveo:fmt-maven-plugin:format
