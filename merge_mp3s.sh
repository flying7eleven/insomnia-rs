#!/bin/bash
find . -maxdepth 1 -iname '*.mp3' -print0 | sort -Vz | xargs -0 mp3wrap ../tmp.mp3
