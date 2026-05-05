import React, { useEffect, useRef } from 'react';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import BrowserOnly from '@docusaurus/BrowserOnly';
import Layout from '@theme/Layout';

// ─── Feature card data ────────────────────────────────────────────────────────

const features = [
  {
    icon: '⚡',
    title: 'Simple Workflow',
    description:
      'Three commands: init, add, sync. osmprj handles the rest — downloading, tuning, importing, and tracking state.',
  },
  {
    icon: '🔧',
    title: 'Auto-Tuned Imports',
    description:
      'osmprj inspects your RAM and storage type and picks the right osm2pgsql flags automatically. No manual tuning required.',
  },
  {
    icon: '🔄',
    title: 'Incremental Updates',
    description:
      'After the first import, subsequent syncs apply only the changes since the last run — fast and bandwidth-efficient.',
  },
];

// ─── Sub-components ───────────────────────────────────────────────────────────

function Hero() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className="hero">
      <img
        src="/osmprj/osmprj-logo-big.svg"
        alt="osmprj logo"
        className="hero__logo"
      />
      <h1 className="hero__title">{siteConfig.title}</h1>
      <p className="hero__subtitle">{siteConfig.tagline}</p>
      <div className="hero__buttons">
        <Link className="button button--primary button--lg" to="/docs/intro">
          Get Started
        </Link>
        <Link
          className="button button--secondary button--lg"
          href="https://github.com/travishathaway/osmprj"
        >
          GitHub
        </Link>
      </div>
    </header>
  );
}

function FeatureCards() {
  return (
    <section className="features">
      {features.map(({ icon, title, description }) => (
        <div key={title} className="featureCard">
          <div className="featureCard__icon">{icon}</div>
          <div className="featureCard__title">{title}</div>
          <p className="featureCard__description">{description}</p>
        </div>
      ))}
    </section>
  );
}

function Demo() {
  return (
    <section className="demo">
      <h2 className="demo__title">See it in action</h2>
      <div className="demo__player">
        {/*
         * The asciinema player must be rendered client-side only — it accesses
         * browser APIs that are unavailable during SSR/build.
         *
         * NOTE: The src path is baseUrl-prefixed. If baseUrl ever changes from
         * "/osmprj/", update "/osmprj/demo.cast" below to match.
         * Replace docs/static/demo.cast with your real recording when ready.
         */}
        <BrowserOnly>
          {() => {
            // asciinema-player is a vanilla JS library — no React component.
            // We use a ref + useEffect to mount it into a DOM node.
            function AsciinemaEmbed() {
              const containerRef = useRef<HTMLDivElement>(null);
              useEffect(() => {
                if (!containerRef.current) return;
                // eslint-disable-next-line @typescript-eslint/no-var-requires
                require('asciinema-player/dist/bundle/asciinema-player.css');
                // eslint-disable-next-line @typescript-eslint/no-var-requires
                const { create } = require('asciinema-player');
                const player = create(
                  '/osmprj/osmprj.cast',
                  containerRef.current,
                  {
                    cols: 100,
                    rows: 28,
                    autoPlay: false,
                    loop: false,
                    speed: 1,
                    theme: 'monokai',
                    fit: 'width',
                  }
                );
                return () => player.dispose();
              }, []);
              return <div ref={containerRef} />;
            }
            return <AsciinemaEmbed />;
          }}
        </BrowserOnly>
      </div>
    </section>
  );
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export default function Home(): React.JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={siteConfig.title}
      description={siteConfig.tagline}
    >
      <main>
        <Hero />
        <FeatureCards />
        <Demo />
      </main>
    </Layout>
  );
}
