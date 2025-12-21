/**
 * GameVault Documentation Sidebar
 *
 * This configuration file defines the sidebar navigation
 * for the Docusaurus documentation site.
 */

module.exports = {
  docs: [
    'intro',
    'getting-started',
    'configuration',
    'usage',
    'settings',
    'faq',
    {
      type: 'category',
      label: 'API Reference',
      items: [
        'api/overview',
        'api/endpoints',
        'api/examples',
      ],
    },
  ],
};
