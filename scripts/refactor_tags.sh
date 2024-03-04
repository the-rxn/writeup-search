#! /bin/sh
jq '.[].tags |= join(",")' writeups_full.json > writeups_full_tags_clean.json
