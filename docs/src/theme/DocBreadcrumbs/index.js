import React from 'react';
import DocBreadcrumbs from '@theme-original/DocBreadcrumbs';
import EditPage from './../../EditPage';

export default function DocBreadcrumbsWrapper(props) {
  return (
    <div className="breadcrumbs">
      <DocBreadcrumbs {...props} />
      <EditPage />
    </div>
  );
}
