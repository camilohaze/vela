# Vela Website Infrastructure

This document explains the complete website setup for Vela, including both the technical documentation site and the marketing website.

## Architecture Overview

Vela has a dual-website setup:

1. **Technical Documentation** (`docs.velalang.org`): Built with mdBook
2. **Marketing Website** (`velalang.org`): Built with Docusaurus

## Directory Structure

```
vela/
├── docs/                          # Technical documentation (mdBook)
│   ├── book.toml                 # mdBook configuration
│   ├── src/                      # Documentation source files
│   └── book/                     # Built documentation (generated)
│
├── website/                       # Marketing website (Docusaurus)
│   ├── docs/                     # Marketing docs
│   ├── blog/                     # Blog posts
│   ├── src/                      # React components
│   ├── static/                   # Static assets
│   ├── package.json              # Dependencies
│   ├── docusaurus.config.js      # Site configuration
│   └── CNAME                     # Domain configuration
│
└── .github/workflows/            # CI/CD pipelines
    ├── deploy-docs.yml          # Documentation deployment
    └── deploy-website.yml       # Marketing site deployment
```

## Deployment Process

### Technical Documentation (mdBook)

- **Source**: `docs/` directory
- **Build Tool**: mdBook with custom themes and preprocessors
- **Domain**: `docs.velalang.org`
- **Deployment**: GitHub Actions on push to `main`
- **Preview**: PR comments with preview links

### Marketing Website (Docusaurus)

- **Source**: `website/` directory
- **Build Tool**: Docusaurus with React components
- **Domain**: `velalang.org`
- **Deployment**: GitHub Actions on push to `main`
- **Preview**: PR comments with preview links

## Cross-Site Navigation

The websites are designed to work together:

- Marketing site links to technical docs
- Technical docs link back to marketing site
- Unified branding and navigation
- Consistent domain structure

## Development Workflow

### Local Development

```bash
# Documentation site
cd docs
mdbook serve

# Marketing site
cd website
npm install
npm start
```

### Production Deployment

Both sites deploy automatically on push to `main` branch via GitHub Actions.

## Domain Configuration

- `velalang.org` → Marketing website (Docusaurus)
- `docs.velalang.org` → Technical documentation (mdBook)

Both domains are configured with CNAME records pointing to GitHub Pages.

## Maintenance

### Updating Documentation
1. Edit files in `docs/src/`
2. Push to `main` branch
3. Automatic deployment via GitHub Actions

### Updating Marketing Site
1. Edit files in `website/`
2. Push to `main` branch
3. Automatic deployment via GitHub Actions

### Adding Blog Posts
1. Create new `.md` files in `website/blog/`
2. Follow the naming convention: `YYYY-MM-DD-title.md`

## Performance & SEO

- Both sites optimized for performance
- SEO-friendly URLs and meta tags
- Fast loading with optimized assets
- Mobile-responsive design
- Search functionality enabled

## Monitoring & Analytics

- GitHub Pages provides basic analytics
- Error monitoring via GitHub Actions logs
- Performance monitoring can be added with third-party tools

## Contributing

See individual README files in `docs/` and `website/` for detailed contribution guidelines.