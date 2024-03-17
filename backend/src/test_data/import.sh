#! /bin/bash

mongoimport --host "$1":"$2" --db test_db --collection Post --type json --file ./src/test_data/posts.json --jsonArray