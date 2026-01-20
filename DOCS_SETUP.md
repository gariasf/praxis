# Documentation Setup Guide

This guide explains how to build and deploy the Praxis course documentation site.

## Quick Start

### Prerequisites

- Python 3.7 or later
- pip (Python package manager)

### Local Development

1. **Install dependencies:**
   ```bash
   pip install -r requirements-docs.txt
   ```

2. **Start development server:**
   ```bash
   mkdocs serve
   ```

3. **Open in browser:**
   ```
   http://localhost:8000
   ```

The site will auto-reload when you make changes to files.

## Building for Production

Build the static site:

```bash
mkdocs build
```

Output will be in the `site/` directory.

### Build Options

```bash
# Build with strict mode (fail on warnings)
mkdocs build --strict

# Clean build (remove previous build)
mkdocs build --clean

# Build to custom directory
mkdocs build --site-dir custom_site/
```

## GitHub Pages Deployment

### Automatic Deployment (Recommended)

The site automatically deploys to GitHub Pages when you push to `main`:

1. GitHub Actions workflow (`.github/workflows/docs.yml`) runs
2. Site is built using MkDocs
3. Published to GitHub Pages

**Setup:**

1. Go to repository **Settings** → **Pages**
2. Source: **GitHub Actions**
3. Workflow will run automatically on push

### Manual Deployment

Deploy manually using MkDocs:

```bash
mkdocs gh-deploy
```

This builds the site and pushes to the `gh-pages` branch.

## Configuration

### Main Config: `mkdocs.yml`

Key sections:

```yaml
site_name: Praxis Game Engine Course
site_url: https://yourusername.github.io/praxis/

theme:
  name: material
  features:
    - navigation.tabs         # Top navigation tabs
    - content.tabs.link       # Sync language tabs
    - content.code.copy       # Copy code buttons

markdown_extensions:
  - pymdownx.tabbed:          # Multi-language tabs
      alternate_style: true
  - pymdownx.superfences      # Code blocks with syntax highlighting
```

### Custom Styling

- **CSS**: `docs/stylesheets/extra.css`
- **JavaScript**: `docs/javascripts/extra.js`

## Site Features

### Multi-Language Code Tabs

Readers can switch between Rust, C++, C#, and Pseudocode:

````markdown
=== "Rust"
    ```rust
    fn example() { }
    ```

=== "C++"
    ```cpp
    void Example() { }
    ```

=== "C#"
    ```csharp
    void Example() { }
    ```
````

### Difficulty Badges

```html
<span class="difficulty-badge difficulty-beginner">Beginner</span>
```

### Feature Grids

```html
<div class="feature-grid">
  <div class="feature-card">
    <h3>Title</h3>
    <p>Description</p>
  </div>
</div>
```

### Admonitions

```markdown
!!! tip "Helpful Hint"
    Use keyboard shortcuts for faster navigation!

!!! warning "Important"
    Be careful with this operation.
```

## Content Structure

```
docs/
├── index.md                    # Home page
├── course/                     # Main course content
│   ├── index.md
│   ├── CURRICULUM.md
│   ├── CODE_EXAMPLES.md
│   ├── examples/              # Detailed examples with tabs
│   ├── patterns/              # Design patterns
│   ├── exercises/             # Practice exercises
│   ├── projects/              # Complete projects
│   └── comparisons/           # Engine comparisons
├── getting-started/           # Installation guides
├── guides/                    # Task-oriented tutorials
├── concepts/                  # Theoretical foundations
├── reference/                 # API documentation
├── learning-paths/            # Structured progressions
└── editor/                    # Editor documentation
```

## Updating Content

### Adding a New Page

1. **Create markdown file:**
   ```bash
   touch docs/course/examples/new-example.md
   ```

2. **Add to navigation in `mkdocs.yml`:**
   ```yaml
   nav:
     - Course:
       - Examples:
         - New Example: course/examples/new-example.md
   ```

3. **Write content with tabs:**
   Use the multi-language tab format for code examples.

### Adding a New Section

1. Create directory: `docs/new-section/`
2. Add index page: `docs/new-section/index.md`
3. Update `mkdocs.yml` navigation
4. Add cross-references from related pages

## Testing

### Check Locally

```bash
# Start server and browse all pages
mkdocs serve

# Test building
mkdocs build --strict

# Check for broken links (manual)
# Browse all pages and check links work
```

### Pre-Deployment Checklist

- [ ] All pages render correctly
- [ ] Code tabs work and sync
- [ ] Search finds expected content
- [ ] Images load properly
- [ ] Internal links work
- [ ] External links are valid
- [ ] Mobile responsive (resize browser)
- [ ] Keyboard shortcuts work

## Troubleshooting

### "Module not found" errors

```bash
pip install --upgrade -r requirements-docs.txt
```

### Tabs not rendering

Check `mkdocs.yml`:
```yaml
markdown_extensions:
  - pymdownx.tabbed:
      alternate_style: true
```

### Search not working

Rebuild search index:
```bash
mkdocs build --clean
```

### GitHub Pages not updating

1. Check Actions tab for workflow status
2. Verify `docs.yml` workflow is enabled
3. Check repository Settings → Pages settings
4. Clear browser cache

### Styles not applying

1. Clear browser cache (Ctrl+F5)
2. Check `extra.css` path in `mkdocs.yml`
3. Verify file exists in `docs/stylesheets/`

## Maintenance

### Update Dependencies

```bash
# Check for updates
pip list --outdated

# Update all
pip install --upgrade -r requirements-docs.txt

# Update specific package
pip install --upgrade mkdocs-material
```

### Monitor Site Health

- Check GitHub Actions for build failures
- Test site after major updates
- Review analytics (if configured)
- Fix broken links regularly

## Advanced Features

### Analytics

Add Google Analytics in `mkdocs.yml`:

```yaml
extra:
  analytics:
    provider: google
    property: G-XXXXXXXXXX
```

### Custom Domain

1. Add `CNAME` file to `docs/`:
   ```
   docs.praxisengine.com
   ```

2. Configure DNS:
   ```
   CNAME docs.praxisengine.com yourusername.github.io
   ```

3. Enable HTTPS in GitHub Pages settings

### Versioning

Use `mike` for version management:

```bash
pip install mike

# Deploy version
mike deploy 1.0 latest --update-aliases

# Set default version
mike set-default latest

# List versions
mike list
```

## Resources

- [MkDocs Documentation](https://www.mkdocs.org/)
- [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/)
- [PyMdown Extensions](https://facelessuser.github.io/pymdown-extensions/)
- [GitHub Pages Docs](https://docs.github.com/en/pages)

## Support

- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: Questions and community support
- Documentation: See `docs/DOCS_README.md` for detailed info

## License

Documentation is licensed under CC BY 4.0.
Code examples follow the main repository license.
