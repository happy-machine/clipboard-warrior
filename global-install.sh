#!/bin/bash
mv target/release/clip /usr/local/bin/clip
mv clipboarddb.json /usr/local/bin/clipboarddb.json
export PATH=$PATH:/usr/local/bin