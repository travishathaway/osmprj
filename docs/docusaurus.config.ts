import { themes as prismThemes } from 'prism-react-renderer';
import type { Config } from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'osmprj',
  tagline: 'A friendly, modern tool for managing OpenStreetMap data with PostgreSQL',
  favicon: 'favicon.ico',

  url: 'https://travishathaway.github.io',
  baseUrl: '/osmprj/',

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
        src: './img/osmprj-logo-small.svg',
        height: 48,
        width: 48
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
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Travis Hathaway. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['bash', 'toml'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
