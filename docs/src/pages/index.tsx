import React, { useEffect, useRef } from 'react';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import BrowserOnly from '@docusaurus/BrowserOnly';
import Layout from '@theme/Layout';
import CodeBlock from '@theme/CodeBlock';
import { Database, Map, Layers } from 'lucide-react';

// ─── Feature card data ────────────────────────────────────────────────────────

const features = [
  {
    num: '01',
    icon: <Database size={22} strokeWidth={1.75} />,
    title: 'Project-based workflow',
    description: (
      <>
        Initialise, save, and check in{' '}
        <code>osmprj.toml</code> alongside your code. Share imports the way
        you share <code>Cargo.toml</code>.
      </>
    ),
  },
  {
    num: '02',
    icon: <Map size={22} strokeWidth={1.75} />,
    title: 'Built on osm2pgsql',
    description:
      'A wrapper, not a replacement. Auto-tunes flags for your hardware and initialises replication so subsequent syncs are incremental.',
  },
  {
    num: '03',
    icon: <Layers size={22} strokeWidth={1.75} />,
    title: 'Nine built-in themes',
    description: (
      <>
        Ship with shortbread, pgosm, osmcarto and more. Add your own by
        extending <code>OSMPRJ_THEME_PATH</code>.
      </>
    ),
  },
];

// ─── Sub-components ───────────────────────────────────────────────────────────

function Hero() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className="hero">
      <div className="hero__inner">
        <div className="hero__logo">
          <img
            src="img/osmprj-logo-big.png"
            alt="osmprj logo"
          />
        </div>
        <div className="hero__text">
          <div className="hero__eyebrow">version · 0.2.0 </div>
          <p className="hero__tag">{siteConfig.tagline}</p>
          <div className="hero__btns">
            <Link className="btn btn--primary" to="/docs/getting-started">
              Get Started →
            </Link>
            <a
              className="btn btn--secondary"
              href="https://github.com/travishathaway/osmprj"
              target="_blank"
              rel="noopener noreferrer"
            >
              ★ GitHub
            </a>
          </div>
        </div>
      </div>
    </header>
  );
}

function FeatureCards() {
  return (
    <section className="features">
      <div className="features__inner">
        <div className="features__head">
          <h2 className="features__title" style={{textAlign: "center"}}>Install</h2>
        </div>
        <div className="term features__install_card">
          <div className="term__bar">
            <div className="term__dots">
              <span />
              <span />
              <span />
            </div>
            <span>~/</span>
            <span>term</span>
          </div>
          <CodeBlock language="bash" title="">
            curl -fsSL https://osmprj.dev/install.sh | bash
          </CodeBlock>
        </div>
        <div className="features__install_card__extra_info">
          Want even more installation methods? <Link to="/docs/getting-started"><strong>See here</strong></Link>.
        </div>
        <div className="features__head">
          <div className="features__eyebrow">▸ what it does</div>
          <h2 className="features__title">Project management for your OSM data</h2>
        </div>
        <div className="features__grid">
          {features.map(({ num, icon, title, description }) => (
            <div key={num} className="feature">
              <div className="feature__corner">{num}</div>
              <div className="feature__icon">{icon}</div>
              <h3 className="feature__title">{title}</h3>
              <p className="feature__desc">{description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

const terminalDemoText = `# 1. Initialize your project
osmprj init --db postgres://user@host:5432/db
Created osmprj.toml

# 2. Add sources to your project and choose a theme
osmprj add bremen --theme shortbread
...
osmprj add hamburg --theme pgosm
...

# 3. Sync your project by downloading importing to PostgreSQL
osmprj sync
...
🌐  Sync complete. 0 sources updated, 2 sources newly imported.`;

function TerminalDemo() {
  return (
    <section className="demo">
      <div className="demo__inner">
        <h2 className="demo__title">See it in action</h2>
        <p className="demo__subtitle">
          Create a database with various Geofabrik regions in three commands.
        </p>
        <div className="term">
          <div className="term__bar">
            <div className="term__dots">
              <span />
              <span />
              <span />
            </div>
            <span>~/projects/osmprj</span>
            <span>term</span>
          </div>
          <div className="demo__code_block">
            <CodeBlock language="bash" title="">
              {terminalDemoText}
            </CodeBlock>
          </div>
        </div>
      </div>
    </section>
  );
}

function AsciinemaSection() {
  return (
    <section className="demo" style={{ paddingTop: 40 }}>
      <div className="demo__inner">
        <h2 className="demo__title">Full walkthrough</h2>
        <p className="demo__subtitle">
          See the entire process of initializing and syncing an osmprj project.
        </p>
        <div className="demo__player">
          {/*
           * The asciinema player must be rendered client-side only — it accesses
           * browser APIs that are unavailable during SSR/build.
           */}
          <BrowserOnly>
            {() => {
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
                      theme: 'osmprj',
                      fit: false,
                      terminalFontSize: 'medium',
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
      </div>
    </section>
  );
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export default function Home(): React.JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout title={siteConfig.title} description={siteConfig.tagline}>
      <main>
        <Hero />
        <FeatureCards />
        <TerminalDemo />
        <AsciinemaSection />
      </main>
    </Layout>
  );
}
