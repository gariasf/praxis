# Architecture Decision Trees

This directory contains interactive decision trees to help you make informed architectural choices when building game engines. Each decision tree explores a common design question, presents trade-offs, and guides you toward the best choice for your specific needs.

## Available Decision Trees

1. **[ECS vs Inheritance](ecs-vs-inheritance.md)** - Should I use Entity Component System or traditional inheritance hierarchies?
2. **[Forward vs Deferred Rendering](forward-vs-deferred-rendering.md)** - Which rendering pipeline should I choose?
3. **[Scene Graphs vs Flat Entity Lists](scene-graphs-vs-flat-lists.md)** - How should I organize my scene objects?
4. **[Custom Math vs Libraries](custom-math-vs-libraries.md)** - Should I write my own math library or use an existing one?
5. **[Multithreading Strategies](multithreading-strategies.md)** - How should I approach parallelism in my engine?
6. **[Asset Loading Approaches](asset-loading-approaches.md)** - Synchronous, asynchronous, or streaming asset loading?

## How to Use These Decision Trees

Each decision tree:
- Starts with a specific design question
- Presents key factors to consider (project type, language, performance needs, etc.)
- Provides branching paths based on your answers
- Explains pros and cons of each option
- Offers recommendations with rationale

**Navigate the trees** by following the paths that match your project requirements. The trees use a flowchart-style format with decision points marked as questions and outcomes as recommendations.

## Contributing

If you identify common architectural decisions not covered here, consider proposing additions that follow the existing format and provide balanced, educational guidance.
