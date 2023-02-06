#!/bin/bash
mkdir -p /usr/local/bin/clip
mv ./clip /usr/local/bin/clip/clip
mv ./clipboarddb.json /usr/local/bin/clip/clipboarddb.json
echo 'export PATH=/usr/local/bin:$PATH' >> ~/.bash_profile
echo 'alias clip="cd /usr/local/bin/clip && ./clip"' >> ~/.bash_profile
source ~/.bash_profile