#!/bin/bash
cd /Users/proxikal/dev/projects/atlas
./atlas-dev --server --debug 2>&1 | tee /tmp/atlas-debug.log
