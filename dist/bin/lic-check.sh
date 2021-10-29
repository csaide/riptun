#!/bin/bash

# (c) Copyright 2021 Christian Saide
# SPDX-License-Identifier: MIT

function list_files_missing_lic() {
    find . \
        -not -path './.cargo/*' \
        -not -path './target/*' \
        -not -path './.git/*' \
        -not -path './.vscode/*' \
        -not -path './dist/tests/*' \
        -not -path './dist/docker/development/config' \
        -not -path './dist/docker/development/.bashrc' \
        -not -name 'root.tar.gz' \
        -not -name .gitignore \
        -not -name .dockerignore \
        -not -name LICENSE \
        -not -name README.md \
        -not -name rust-toolchain \
        -not -name codecov.yaml \
        -not -name '*.json' \
        -not -name '*.lock' \
        -not -name '*.toml' \
        -not -name '*.pem' \
        -not -name '*.srl' \
        -type f | xargs grep -L 'SPDX-License-Identifier: MIT'
}

function count_files_missing_lic() {
    list_files_missing_lic | wc -l
}

function list_files_missing_copy() {
    find . \
        -not -path './.cargo/*' \
        -not -path './target/*' \
        -not -path './.git/*' \
        -not -path './.vscode/*' \
        -not -path './dist/tests/*' \
        -not -path './dist/docker/development/config' \
        -not -path './dist/docker/development/.bashrc' \
        -not -name 'root.tar.gz' \
        -not -name .gitignore \
        -not -name .dockerignore \
        -not -name LICENSE \
        -not -name rust-toolchain \
        -not -name codecov.yaml \
        -not -name '*.json' \
        -not -name '*.lock' \
        -not -name '*.toml' \
        -not -name '*.pem' \
        -not -name '*.srl' \
        -type f | xargs grep -L -E '(&copy;|\(c\)) Copyright 2021 Christian Saide'
}

function count_files_missing_copy() {
    list_files_missing_copy | wc -l
}

if [ $(count_files_missing_lic) -ne 0 ]; then
    cat <<EOF
There are files missing the 'SPDX-License-Identifier: MIT' license identifier.

Files:
$(list_files_missing_lic)
EOF
    exit 1
fi

if [ $(count_files_missing_copy) -ne 0 ]; then
    cat <<EOF
There are files missing the '&copy;|(c) Copyright 2021 Christian Saide' copyright identifier.

Files:
$(list_files_missing_copy)
EOF
    exit 1
fi

echo "All files have correct copyright and license identifiers."
exit 0
