// @ts-check
import { themes as prismThemes } from 'prism-react-renderer';

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'GameVault',
  tagline: 'A portable game library manager for Windows',
  favicon: 'img/favicon.ico',

  url: 'https://gamevault.dev',
  baseUrl: '/',

  organizationName: 'gamevault',
  projectName: 'gamevault',

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          path: '.',
          include: ['**/*.md', '**/*.mdx'],
          exclude: ['**/node_modules/**', 'FEATURE_PLAN.md', 'FEATURE_PLAN_EDIT_AND_DETAILS.md', 'ENRICHMENT_ARCHITECTURE.md', 'PORTABLE_EXECUTABLE_PLAN.md', 'CONFIGURATION_PAGE_PLAN.md'],
          sidebarPath: './sidebars.js',
          routeBasePath: '/',
          editUrl: 'https://github.com/gamevault/gamevault/tree/main/docs/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      image: 'img/gamevault-social-card.png',
      navbar: {
        title: 'GameVault',
        logo: {
          alt: 'GameVault Logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'docs',
            position: 'left',
            label: 'Documentation',
          },
          {
            to: '/api/overview',
            label: 'API',
            position: 'left',
          },
          {
            to: '/architecture/overview',
            label: 'Architecture',
            position: 'left',
          },
          {
            href: 'https://github.com/gamevault/gamevault',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Documentation',
            items: [
              {
                label: 'Getting Started',
                to: '/getting-started',
              },
              {
                label: 'Configuration',
                to: '/configuration',
              },
              {
                label: 'API Reference',
                to: '/api/overview',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'GitHub',
                href: 'https://github.com/gamevault/gamevault',
              },
              {
                label: 'Issues',
                href: 'https://github.com/gamevault/gamevault/issues',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'Architecture',
                to: '/architecture/overview',
              },
              {
                label: 'Development',
                to: '/development',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} GameVault. Built with Docusaurus.`,
      },
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.dracula,
        additionalLanguages: ['rust', 'toml', 'powershell', 'bash', 'json'],
      },
      colorMode: {
        defaultMode: 'dark',
        disableSwitch: false,
        respectPrefersColorScheme: true,
      },
    }),
};

export default config;
