#!/bin/sh

# This builds the ibc-rs.informal.systems docs using docusaurus.

COMMIT=$(git rev-parse HEAD)

chmod +x ./pre.sh ./post.sh
mkdir -p ~/versioned_docs  ~/versioned_sidebars
for version in $(jq -r .[] versions.json); do
    echo "building docusaurus $version docs"
    git clean -fdx && git reset --hard && git checkout release/$version.x
    ./pre.sh
    npm ci && npm run docusaurus docs:version $version
    mv ./versioned_docs/* ~/versioned_docs/
    mv ./versioned_sidebars/* ~/versioned_sidebars/
done

echo "building docusaurus main docs"
(git clean -fdx && git reset --hard && git checkout $COMMIT)
mv ~/versioned_docs ~/versioned_sidebars .
npm ci && npm run build
mv build ~/output

# echo "setup domain"
# echo $DOCS_DOMAIN > ~/output/CNAME