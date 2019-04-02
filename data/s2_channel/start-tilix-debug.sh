#!/usr/bin/env bash

CURR_DIR=$(dirname $0)

tilix -a app-new-window -t "Debug ENV" -w $CURR_DIR -x "./_start-tilix-debug.sh"