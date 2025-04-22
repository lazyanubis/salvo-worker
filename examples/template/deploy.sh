#!/bin/bash

cargo clippy

npx wrangler deploy

echo ''
echo '========= deploy successful. ========='
echo ''
