import React from "react";
import Link from "@docusaurus/Link";
import { useState, useEffect } from 'react';
import EditIcon from './../static/img/edit.svg';


export default function EditPage() {
  const [currentPageName, setCurrentPageName] = useState('');

  useEffect(() => {
    let pagePath = window.location.pathname.slice(1);

    if (pagePath.startsWith('developers/architecture/')) {
      pagePath = pagePath.replace('developers/architecture/', 'architecture/')
    }

    pagePath = "docs/" + pagePath;

    const pageMappings = {
      'docs/': 'docs/learn/intro',
      'docs/developers/intro/contributing': 'CONTRIBUTING',
      'docs/developers/migrations/changelog': 'CHANGELOG',
      'docs/developers/architecture': 'docs/architecture/README',
    };

    if (pagePath in pageMappings) {
      pagePath = pageMappings[pagePath];
    }

    setCurrentPageName(pagePath);
  }, []);

  const githubBaseUrl = "https://github.com/cosmos/ibc-rs";
  const githubUrl = `${githubBaseUrl}/edit/main/${currentPageName}.md`;

  return (
    <Link href={githubUrl} className="edit-page">
      <EditIcon className="w-5 h-5 mr-2 fill-current" />
      <div >Edit this page</div>
    </Link>
  );
}
