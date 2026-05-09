import React, { useEffect, useRef } from 'react';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import BrowserOnly from '@docusaurus/BrowserOnly';
import Layout from '@theme/Layout';
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
            src="/osmprj/img/osmprj-logo-big.png"
            alt="osmprj logo"
          />
        </div>
        <div className="hero__text">
          <div className="hero__eyebrow">alpha · 0.1.0</div>
          <p className="hero__tag">{siteConfig.tagline}</p>
          <div className="hero__btns">
            <Link className="btn btn--primary" to="/docs/intro">
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
          <div className="features__eyebrow">▸ what it does</div>
          <h2 className="features__title">Three commands. One project file.</h2>
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

function TerminalDemo() {
  return (
    <section className="demo">
      <div className="demo__inner">
        <h2 className="demo__title">See it in action</h2>
        <p className="demo__subtitle">
          Spin up a fresh database with Germany&apos;s OSM data in four commands.
        </p>
        <div className="term">
          <div className="term__bar">
            <div className="term__dots">
              <span />
              <span />
              <span />
            </div>
            <span>~/projects/germany-osm</span>
            <span>bash</span>
          </div>
          <pre>{/* prettier-ignore */}<span className="c"># 1. Create an osmprj.toml in the current directory</span>{"\n"}<span className="k">$</span>{" osmprj "}<span className="p">init</span>{" --db "}<span className="s">&quot;postgres://postgres@localhost/osm&quot;</span>{"\n\n"}<span className="c"># 2. Register a Geofabrik region</span>{"\n"}<span className="k">$</span>{" osmprj "}<span className="p">add</span>{" germany --theme "}<span className="s">shortbread</span>{"\n  "}<span className="ok">✓</span>{" schema "}<span className="s">germany</span>{" registered\n\n"}<span className="c"># 3. Check what will be synced</span>{"\n"}<span className="k">$</span>{" osmprj "}<span className="p">status</span>{"\n  database:  postgres://postgres@localhost/osm  "}<span className="ok">✓ connected</span>{"\n\n  source   schema   status\n  ------   ------   ------\n  germany  germany  "}<span className="p">✗</span>{"  — run "}<span className="s">&apos;osmprj sync&apos;</span>{" to import\n\n"}<span className="c"># 4. Download, tune, and import</span>{"\n"}<span className="k">$</span>{" osmprj "}<span className="p">sync</span>{"\n  "}<span className="ok">→</span>{" downloading germany-latest.osm.pbf  ["}<span className="ok">██████████</span>{"] 4.2 GB\n  "}<span className="ok">→</span>{" auto-tuning  --cache=12000 --flat-nodes\n  "}<span className="ok">→</span>{" osm2pgsql --create --slim --output=flex\n  "}<span className="ok">✓</span>{" import complete in 28m 14s"}</pre>
        </div>
      </div>
    </section>
  );
}

function AsciinemaSection() {
  return (
    <section className="demo" style={{ paddingTop: 40 }}>
      <div className="demo__inner">
        <h2 className="demo__title" style={{ fontSize: 24, marginBottom: 24 }}>
          Full walkthrough
        </h2>
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
