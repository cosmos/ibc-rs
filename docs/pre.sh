#!/usr/bin/env bash

## Add architecture documentation
rsync -a ./architecture/* ./docs/developers/06-architecture

## Add changelog documentation
cp -r ./../CHANGELOG.md ./docs/developers/07-migrations/01-changelog.md

## Add contributing documentation
cp -r ./../CONTRIBUTING.md ./docs/developers/01-intro/01-contributing.md
