const lightCodeTheme = require("prism-react-renderer/themes/github");
const darkCodeTheme = require("prism-react-renderer/themes/dracula");

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "IBC-rs",
  tagline:
    "Welcome to the IBC-rs documentation!",
  url: "https://ibc-rs.cosmos.network",
  baseUrl: "/",
  onBrokenLinks: "warn",
  onBrokenMarkdownLinks: "warn",
  favicon: "img/favicon.svg",
  trailingSlash: false,
  organizationName: "cosmos",
  projectName: "ibc-rs",
  deploymentBranch: "gh-pages",
  i18n: {
    defaultLocale: "en",
    locales: ["en"],
    localeConfigs: {
      en: {
        label: "English",
      },
    },
  },
  presets: [
    [
      "classic",
      /** @type {import('@docusaurus/preset-classic').Options} */
      {
        theme: {
          customCss: require.resolve("./src/css/custom.css"),
        },    
        docs: {
          sidebarPath: require.resolve("./sidebars.js"),
          routeBasePath: "/",
          lastVersion: "current",
          // versions: {
          //   "v0.42.0": {
          //     path: "v0.42.0",
          //     label: "v0.42.0",
          //   },
          // },
        },
      },
    ],
  ],
  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      image: "img/banner.jpg",
      docs: {
        sidebar: {
          autoCollapseCategories: true,
        },
      },
      navbar: {
        title: "IBC-rs",
        hideOnScroll: false,
        items: [
          { to: '/', label: 'Learn', position: 'left' },
          { to: '/developers/intro', label: 'Developers', position: 'left' },
          { href: "https://informal.systems/blog/category/all", label: "Blog", position: "left" },
          {
            type: "docsVersionDropdown",
            position: "right",
            dropdownActiveClassDisabled: true,
          },
          {
            href: "https://github.com/cosmos/ibc-rs/",
            html: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" class="github-icon">
            <path fill-rule="evenodd" clip-rule="evenodd" d="M12 0.300049C5.4 0.300049 0 5.70005 0 12.3001C0 17.6001 3.4 22.1001 8.2 23.7001C8.8 23.8001 9 23.4001 9 23.1001C9 22.8001 9 22.1001 9 21.1001C5.7 21.8001 5 19.5001 5 19.5001C4.5 18.1001 3.7 17.7001 3.7 17.7001C2.5 17.0001 3.7 17.0001 3.7 17.0001C4.9 17.1001 5.5 18.2001 5.5 18.2001C6.6 20.0001 8.3 19.5001 9 19.2001C9.1 18.4001 9.4 17.9001 9.8 17.6001C7.1 17.3001 4.3 16.3001 4.3 11.7001C4.3 10.4001 4.8 9.30005 5.5 8.50005C5.5 8.10005 5 6.90005 5.7 5.30005C5.7 5.30005 6.7 5.00005 9 6.50005C10 6.20005 11 6.10005 12 6.10005C13 6.10005 14 6.20005 15 6.50005C17.3 4.90005 18.3 5.30005 18.3 5.30005C19 7.00005 18.5 8.20005 18.4 8.50005C19.2 9.30005 19.6 10.4001 19.6 11.7001C19.6 16.3001 16.8 17.3001 14.1 17.6001C14.5 18.0001 14.9 18.7001 14.9 19.8001C14.9 21.4001 14.9 22.7001 14.9 23.1001C14.9 23.4001 15.1 23.8001 15.7 23.7001C20.5 22.1001 23.9 17.6001 23.9 12.3001C24 5.70005 18.6 0.300049 12 0.300049Z" fill="currentColor"/>
            </svg>
            `,
            position: "right",
          },
        ],
      },
      footer: {
        links: [
          {
            items: [
              {
                html: `<a href="https://cosmos.network"><img src="/img/logo-cosmos.svg" alt="Cosmos Logo"></a>`,
              },
              {
                html: `<a href="https://informal.systems"><img  style="margin-top: 2rem" src="/img/logo-informal.svg" alt="Informal Systems Logo"></a>`,
              },
            ],
          },
          {
            title: "Documentation",
            items: [
              {
                label: "CometBFT",
                href: "https://docs.cometbft.com",
              },
              {
                label: "Hermes",
                href: "https://hermes.informal.systems/",
              },
              {
                label: "IBC Go",
                href: "https://ibc.cosmos.network",
              },
            ],
          },
          {
            title: "Community",
            items: [
              {
                label: "GitHub",
                href: "https://github.com/cosmos/ibc-rs",
              },
              {
                label: "Twitter",
                href: "https://twitter.com/informalinc",
              },
              {
                label: "Youtube",
                href: "https://www.youtube.com/playlist?list=PLlx_69qFQKlSTc9FgRVU5awwCsziLeeQf",
              },
              // We are not using Discord for now. Keeping it for future reference.
              // {
              //   label: "Discord",
              //   href: "https://discord.gg/MkvKh6jpsA",
              // },
            ],
          },
        ],
        copyright: `<p>The development of the IBC-rs is led primarily by <a href="https://informal.systems">Informal Systems Inc</a>. Funding for this development comes primarily from the Interchain Foundation, a Swiss non-profit.</p>`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
        additionalLanguages: ["rust"],
      },
      // TODO: add algolia search
      algolia: {
        appId: "QLS2QSP47E",
        apiKey: "4d9feeb481e3cfef8f91bbc63e090042",
        indexName: "cosmos_network",
        contextualSearch: false,
      },
    }),
  themes: ["@you54f/theme-github-codeblock"],
  plugins: [
    async function myPlugin(context, options) {
      return {
        name: "docusaurus-tailwindcss",
        configurePostCss(postcssOptions) {
          postcssOptions.plugins.push(require("postcss-import"));
          postcssOptions.plugins.push(require("tailwindcss/nesting"));
          postcssOptions.plugins.push(require("tailwindcss"));
          postcssOptions.plugins.push(require("autoprefixer"));
          return postcssOptions;
        },
      };
    },
    [
      "@docusaurus/plugin-client-redirects",
      {
        fromExtensions: ["html"],
        toExtensions: ["html"],
        redirects: [
          {
            from: ["/master", "/main", "/learn"],
            to: "/",
          },
        ],
      },
    ],
  ],
};

module.exports = config;
