import React from "react";
import Link from "@docusaurus/Link";
import EditIcon from './../static/img/edit.svg';


export default function EditPage() {

  let currentPageName = window.location.pathname.slice(1);

  if (currentPageName.startsWith('developers/architecture/')) {
    currentPageName = currentPageName.replace('developers/architecture/', 'architecture/')
  }

  currentPageName = "docs/" + currentPageName;

  const pageMappings = {
    'docs/': 'docs/learn/intro',
    'docs/developers/intro/contributing': 'CONTRIBUTING',
    'docs/developers/migrations/changelog': 'CHANGELOG',
    'docs/developers/architecture': 'docs/architecture/README',
  };

  if (currentPageName in pageMappings) {
    currentPageName = pageMappings[currentPageName];
  }

  const githubBaseUrl = "https://github.com/cosmos/ibc-rs";
  const githubUrl = `${githubBaseUrl}/edit/main/${currentPageName}.md`;


  return (
    <Link href={githubUrl} className="edit-page">
      <EditIcon className="w-5 h-5 mr-2 fill-current" />
      <div >Edit this page</div>
    </Link>
  );
}
