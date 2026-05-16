import React, {type ReactNode} from 'react';
import clsx from 'clsx';
import {ThemeClassNames} from '@docusaurus/theme-common';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import type {Props} from '@theme/Footer/Layout';
import useBaseUrl from '@docusaurus/useBaseUrl';

export default function FooterLayout({
  style,
  links,
  copyright,
}: Props): ReactNode {
  const logoUrl = useBaseUrl('/img/osmprj-logo-small.svg');
  const { siteConfig } = useDocusaurusContext();

  return (
    <footer
      className={clsx(ThemeClassNames.layout.footer.container, 'footer', {
        'footer--dark': style === 'dark',
      })}>
      <div className="footer__inner">
        {/* Brand block — column 1 */}
        <div className="footer__brand">
          <div className="footer__brand-logo">
            <img src={logoUrl} alt="" />
            <strong>osmprj</strong>
          </div>
          <p className="footer__brand-tagline">
            {siteConfig.tagline}. GPL-3.0 licensed.
          </p>
        </div>

        {/* Link columns — columns 2+ */}
        {links}
      </div>

      {copyright && (
        <div className="footer__copy">{copyright}</div>
      )}
    </footer>
  );
}
