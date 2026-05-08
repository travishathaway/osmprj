import React, { useEffect, useRef } from 'react';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import BrowserOnly from '@docusaurus/BrowserOnly';
import Layout from '@theme/Layout';
import { Highlight, themes } from "prism-react-renderer"

// ─── Feature card data ────────────────────────────────────────────────────────

const features = [
  {
    icon: '💾',
    title: 'Project based workflow',
    description:
      'Initialize and save your project configuration right alongside your code for easier sharing.'
  },
  {
    icon: '🏛️',
    title: 'Solid foundation',
    description:
      'Built on top of the tried and true osm2pgsql for reliable imports and performance.'
  },
  {
    icon: '🎨',
    title: 'Builtin themes',
    description:
      'Easily select default themes for a variety database schema layouts that can later be synced.'
  },
];

// ─── Sub-components ───────────────────────────────────────────────────────────

function Hero() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className="hero">
      <div className="col hero__banner-image">
        <img
            src="/osmprj/img/osmprj-logo-big.svg"
            alt="osmprj logo"
            className="hero__logo"
        />
      </div>
      <div className="col hero__text">
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className="hero__buttons">
          <Link className="button button--primary button--lg" to="/docs/intro">
            Get Started
          </Link>
          <iframe
              className="hero__gh-star-btn"
            src="https://ghbtns.com/github-btn.html?user=travishathaway&repo=osmprj&type=star&count=true&size=large"
            width={160}
            height={30}
            title="GitHub Stars"
          />
        </div>
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

function ShellCodeBlock(code) {
  return (
      <Highlight code={code.trim()} language="bash" theme={themes.github}>
        {({ className, style, tokens, getLineProps, getTokenProps }) => (
            <pre
                className={className}
                style={{
                  ...style,
                  padding: "1rem",
                  borderRadius: "0.5rem",
                  overflowX: "auto",
                }}
            >
          {tokens.map((line, i) => (
              <div key={i} {...getLineProps({ line })}>
                {line.map((token, key) => (
                    <span key={key} {...getTokenProps({ token })} />
                ))}
              </div>
          ))}
        </pre>
        )}
      </Highlight>
  );
}

function InstallCard() {
  let code = `curl -fsSl https://example.com/install.sh | bash`
  return (
      <section className="install">
        <h2>Install</h2>
        <p>Install</p>
        <ShellCodeBlock code={code} />
      </section>
  )
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
                    cols: 80,
                    rows: 20,
                    autoPlay: false,
                    loop: false,
                    speed: 1.5,
                    theme: 'dracula',
                    fit: false,
                    terminalFontSize: 'medium'
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
        <InstallCard />
        <FeatureCards />
        <Demo />
      </main>
    </Layout>
  );
}
