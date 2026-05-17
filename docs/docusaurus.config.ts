import type { Config } from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import type { PrismTheme } from 'prism-react-renderer';

// osmprj cartographic code theme — dark terminal palette from the design handoff
const osmprjCodeThemeDark: PrismTheme = {
  plain: {
    color: '#ECEFE0',
    backgroundColor: '#1F2410',
  },
  styles: [
    { types: ['comment', 'prolog', 'doctype', 'cdata'], style: { color: '#7A8273', fontStyle: 'italic' } },
    { types: ['punctuation'], style: { color: '#9CA391' } },
    { types: ['keyword', 'selector', 'important', 'atrule'], style: { color: '#ADCC73' } },
    { types: ['property', 'tag', 'boolean', 'number', 'constant', 'symbol', 'deleted'], style: { color: '#ADCC73' } },
    { types: ['string', 'char', 'builtin', 'inserted', 'attr-value'], style: { color: '#E8CB89' } },
    { types: ['operator', 'entity', 'url', 'variable', 'language-css'], style: { color: '#D97742' } },
    { types: ['function', 'class-name'], style: { color: '#ADCC73' } },
    { types: ['regex', 'important'], style: { color: '#E8CB89' } },
    { types: ['italic'], style: { fontStyle: 'italic' } },
    { types: ['bold'], style: { fontWeight: 'bold' } },
  ],
};

// osmprj cartographic code theme — light variant (ink on parchment)
const osmprjCodeThemeLight: PrismTheme = {
  plain: {
    color: '#1F2410',       // ink-900 on
    backgroundColor: '#F4EFE3', // parchment-200 (bg-sunken)
  },
  styles: [
    { types: ['comment', 'prolog', 'doctype', 'cdata'], style: { color: '#7A8273', fontStyle: 'italic' } },
    { types: ['punctuation'], style: { color: '#5C6353' } },            // stone-600
    { types: ['keyword', 'selector', 'important', 'atrule'], style: { color: '#375815' } },  // moss-800
    { types: ['property', 'tag', 'boolean', 'number', 'constant', 'symbol', 'deleted'], style: { color: '#375815' } }, // moss-800
    { types: ['string', 'char', 'builtin', 'inserted', 'attr-value'], style: { color: '#A8772A' } }, // ochre-600
    { types: ['operator', 'entity', 'url', 'variable', 'language-css'], style: { color: '#8C3F1F' } }, // terracotta-700
    { types: ['function', 'class-name'], style: { color: '#4E7A1E' } }, // moss-600
    { types: ['regex', 'important'], style: { color: '#A8772A' } },     // ochre-600
    { types: ['italic'], style: { fontStyle: 'italic' } },
    { types: ['bold'], style: { fontWeight: 'bold' } },
  ],
};

const config: Config = {
  title: 'osmprj',
  tagline: 'A friendly, modern CLI for working with OpenStreetMap data in PostgreSQL',
  favicon: 'favicon.ico',

  url: 'https://osmprj.dev',
  baseUrl: '/',

  organizationName: 'travishathaway',
  projectName: 'osmprj',
  deploymentBranch: 'gh-pages',
  trailingSlash: false,

  onBrokenLinks: 'throw',

  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'warn',
    },
  },

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/travishathaway/osmprj/edit/main/docs/',
        },
        blog: {
          path: "blog",
          blogSidebarTitle: "All Posts"
        },
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/osmprj-logo-big.svg',
    navbar: {
      title: 'osmprj',
      logo: {
        alt: 'osmprj',
        src: 'img/osmprj-logo-small.svg',
        height: 36,
        width: 36
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Docs',
        },
        {
          to: 'blog',
          label: 'Blog',
          position: 'left'
        },
        {
          href: 'https://github.com/travishathaway/osmprj',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Introduction',
              to: '/docs/intro',
            },
            {
              label: 'Getting Started',
              to: '/docs/getting-started',
            },
            {
              label: 'Themes',
              to: '/docs/guides/themes',
            },
            {
              label: 'Configuration',
              to: '/docs/reference/configuration',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/travishathaway/osmprj',
            },
            {
              label: 'Report an Issue',
              href: 'https://github.com/travishathaway/osmprj/issues',
            },
            {
              label: 'Sponsor',
              href: 'https://github.com/sponsors/travishathaway',
            },
            {
              label: 'Blog',
              to: '/blog',
            },
          ],
        },
      ],
      copyright: `© ${new Date().getFullYear()} Travis Hathaway · Built with Docusaurus`,
    },
    prism: {
      theme: osmprjCodeThemeDark,
      darkTheme: osmprjCodeThemeDark,
      additionalLanguages: ['bash', 'toml'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
