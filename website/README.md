# Vela Website

This directory contains the marketing website for Vela, built with [Docusaurus](https://docusaurus.io/).

## Development

### Prerequisites

- Node.js 18.0 or higher
- npm or yarn

### Local Development

```bash
cd website
npm install
npm start
```

The site will be available at `http://localhost:3000`.

### Building

```bash
cd website
npm run build
```

The built site will be in `website/build/`.

## Deployment

The website is automatically deployed to `velalang.org` via GitHub Actions when changes are pushed to the `main` branch.

## Structure

- `docs/` - Documentation pages
- `blog/` - Blog posts
- `src/` - React components and pages
- `static/` - Static assets (images, etc.)
- `docusaurus.config.js` - Site configuration
- `sidebars.js` - Documentation sidebar configuration

## Contributing

1. Make changes to the website content
2. Test locally with `npm start`
3. Create a pull request
4. The site will be automatically deployed on merge to main