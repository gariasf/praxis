# Logging Standards for Praxis Engine

This document outlines the logging standards and best practices implemented throughout the Praxis engine codebase.

## Log Levels

We follow industry-standard log levels with specific use cases for each:

### ERROR

- **Usage**: Unrecoverable errors that will cause functionality issues
- **Examples**:
  - Failed to create Vulkan resources
  - GPU submission failures
  - Missing required hardware capabilities
- **Pattern**: Always include the error details and context

```rust
error!("Failed to create graphics pipeline: {}", e);
```

### WARN

- **Usage**: Recoverable issues, performance concerns, or unexpected but handled conditions
- **Examples**:
  - Suboptimal swapchain state
  - Window events received before initialization
  - Performance degradation
- **Pattern**: Include what action will be taken

```rust
warn!("Swapchain is suboptimal, will recreate on next frame");
```

### INFO

- **Usage**: Major state changes, initialization/shutdown, and periodic status updates
- **Examples**:
  - Application startup/shutdown
  - Graphics context initialization
  - FPS reporting (every 2 seconds)
  - Swapchain recreation
- **Pattern**: High-level summary information

```rust
info!("Graphics context initialization complete in {:?}", init_start.elapsed());
info!("Performance: {:.1} FPS (frames: {}, time: {:.1}s)", fps, frame_count, elapsed);
```

### DEBUG

- **Usage**: Detailed flow information useful for debugging
- **Examples**:
  - Resource creation/destruction
  - State transitions
  - Configuration changes
- **Pattern**: Include relevant parameters and timing

```rust
debug!("Creating swapchain with {} images at {}x{} in {:?}",
       image_count, width, height, duration);
```

### TRACE

- **Usage**: Very detailed information including data values
- **Examples**:
  - Individual frame operations
  - Resource enumeration
  - Step-by-step operation flow
- **Pattern**: Include all relevant data

```rust
trace!("Image {} acquired in {:?}", image_index, acquire_start.elapsed());
trace!("Frame {} rendering complete", self.frame_count);
```

## Best Practices

1. **Be Consistent**: Use the same patterns throughout the codebase
2. **Include Timing**: Always measure and log duration for significant operations
3. **Provide Context**: Include relevant parameters and state information
4. **Use Structured Data**: Include dimensions, counts, and identifiers
5. **Avoid Spam**: Use appropriate log levels to avoid overwhelming output
6. **Error Details**: Always log the actual error message, not just "operation failed"
7. **Performance Metrics**: Report aggregate metrics periodically, not every frame
