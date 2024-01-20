#! /bin/bash

mongoimport --host "$1":"$2" --db test_db --collection posts --type json --file ./test_data/posts.json --jsonArray