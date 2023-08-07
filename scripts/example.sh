#!/bin/bash

# script should return list of urls that you would like to keep track of
# this approach allows us to use scripting to obtain the data we want instead of manually writing it for code

jq '.children[0].children[0].children[0]'  | grep href | sed 's/ *"href": "//g;s/",$//g' | uniq | grep http
