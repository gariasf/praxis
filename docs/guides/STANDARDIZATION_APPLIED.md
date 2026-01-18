# Documentation Standardization Applied

This document tracks the standardization applied to technical documentation in `docs/guides/`.

## Standardization Goals

1. **Consistent Structure**: All guides follow the same organizational pattern
2. **Conceptual Clarity**: Each guide explains "why" and "when" to use features
3. **Verified Examples**: Code examples use correct APIs and imports
4. **Cross-References**: Clear navigation between related documents

## Standard Document Structure

### Required Sections (in order)

1. **Title and Overview**
   - Brief description (1-2 paragraphs)
   - Key features bulleted list
   - When to use this feature

2. **Concepts** (if applicable)
   - Theoretical foundation
   - Why this approach
   - Comparison with alternatives

3. **Core Components**
   - Data structures and types
   - Key APIs
   - Configuration options

4. **Implementation**  
   - Step-by-step setup
   - Code examples with full context
   - Integration patterns

5. **Usage Examples**
   - Common use cases
   - Complete working code
   - Best practices

6. **Performance**
   - Cost analysis
   - Optimization strategies
   - Scalability guidelines

7. **Troubleshooting**
   - Common issues and solutions
   - Debug techniques
   - Known limitations

8. **Examples** (cargo commands)
   - Runnable examples from the repo

9. **See Also**
   - Related documentation
   - Cross-references

10. **References** (if applicable)
    - Academic papers
    - External resources

### Code Example Standards

- Always include necessary imports
- Use realistic variable names
- Show complete context (not fragments)
- Include error handling where appropriate
- Comment non-obvious logic

### Cross-Reference Format

Use relative links:
```markdown
See [HDR and Tone Mapping](hdr-tonemapping.md) for details.
```

Not absolute paths or anchors unless necessary.

## Files Standardized

### Rendering Guides (docs/guides/rendering/)

- [x] README.md - Index updated with clear navigation
- [ ] forward-rendering.md - Core concepts added
- [ ] deferred-rendering.md - Performance comparison enhanced
- [ ] hdr-tonemapping.md - Theory section added
- [ ] shadows.md - Algorithm explanation improved
- [ ] bloom.md - Usage patterns clarified
- [ ] post-processing.md - Effect chaining best practices
- [ ] particles.md - Examples verified
- [ ] environment-probes.md - IBL theory expanded
- [ ] advanced-lighting.md - Structured
- [ ] advanced-materials.md - Structured
- [ ] advanced-rendering.md - Structured  
- [ ] frustum-culling.md - Concepts added
- [ ] gpu-culling.md - When-to-use section added
- [ ] lod.md - Theory explained
- [ ] material-instancing.md
- [ ] line-rendering.md
- [ ] line-rendering-quick-ref.md
- [ ] skybox.md
- [ ] cinematic-effects.md
- [ ] ssr.md
- [ ] taa.md

### Animation Guides (docs/guides/animation/)

- [x] README.md - Navigation improved
- [ ] skeletal-basics.md - Already excellent
- [ ] blending.md - Already excellent
- [ ] advanced-features.md - Already excellent
- [ ] skeletal-animation.md
- [ ] quick-start.md
- [ ] quick-reference.md
- [ ] advanced-integration.md

### Other Guides (docs/guides/)

- [ ] README.md - Index
- [ ] animation.md
- [ ] audio.md
- [ ] console.md
- [ ] input.md
- [ ] physics.md
- [ ] rendering.md
- [ ] scripting.md
- [ ] serialization.md
- [ ] spatial-optimization.md
- [ ] terrain.md
- [ ] async-assets.md

## Verification Checklist

For each document:

- [ ] Has clear overview explaining purpose
- [ ] Includes "When to use" section
- [ ] Code examples compile (imports verified)
- [ ] Cross-references use relative links
- [ ] Performance section included
- [ ] Examples section lists cargo commands
- [ ] Troubleshooting section present
- [ ] Follows standard section ordering
- [ ] No line number references
- [ ] Conceptual explanations before code

## Notes

- Line number references: None found in guides (already clean)
- Most rendering and animation guides are already high-quality
- Focus areas: Consistency, cross-references, when-to-use sections
- Examples mostly use correct APIs (minor fixes only)
