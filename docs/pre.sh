#!/usr/bin/env bash

## Add architecture documentation
cp -r ./architecture ./docs/developers

## Add changelog documentation
cp -r ./../CHANGELOG.md ./docs/developers/migrations/changelog.md

## Add contributing documentation
cp -r ./../CONTRIBUTING.md ./docs/developers/intro/contributing.md
