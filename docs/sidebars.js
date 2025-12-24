/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docs: [
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'intro',
        'getting-started',
        'configuration',
        'usage',
        'settings',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      collapsed: false,
      items: [
        'api/overview',
        'api/endpoints',
        'api/examples',
      ],
    },
    {
      type: 'category',
      label: 'Architecture',
      collapsed: false,
      items: [
        'architecture/overview',
        'architecture/backend',
        'architecture/frontend',
        'architecture/database',
        'architecture/embedding',
      ],
    },
    {
      type: 'category',
      label: 'Components',
      collapsed: true,
      items: [
        'components/overview',
        'components/game-card',
        'components/edit-modal',
        'components/adjust-match-modal',
        'components/game-menu',
        'components/settings-modal',
      ],
    },
    {
      type: 'category',
      label: 'Backend Modules',
      collapsed: true,
      items: [
        'backend/overview',
        'backend/handlers',
        'backend/database',
        'backend/scanner',
        'backend/steam',
        'backend/local-storage',
      ],
    },
    'development',
    'faq',
  ],
};

module.exports = sidebars;
