#!/bin/bash
mv ./clip /usr/local/bin/clip
mv ./clipboarddb.json /usr/local/bin/clipboarddb.json
echo 'export PATH=/usr/local/bin:$PATH' >> ~/.bash_profile
echo 'alias clip="cd /usr/local/bin/ && ./clip"' >> ~/.bash_profile
source ~/.bash_profile