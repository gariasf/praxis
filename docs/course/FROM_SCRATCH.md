# Building a Game Engine From Scratch

A comprehensive guide to implementing a minimal 3D game engine from first principles, with parallel implementation tracks for different languages and frameworks.

## Overview

This guide walks you through building a minimal but functional 3D game engine from the ground up. Rather than using an existing engine, you'll implement core systems yourself to deeply understand how game engines work internally.

**What You'll Build:**

- Window creation and management
- Graphics API initialization and basic rendering
- Input handling (keyboard, mouse)
- Game loop with proper timing
- 3D camera system
- Basic mesh rendering
- Simple material/shader system
- Resource loading

**Implementation Tracks:**

Choose one track based on your language preference and learning goals:

- **🦀 Rust + wgpu**: Modern, safe systems programming with cross-platform graphics
- **⚙️ C++ + OpenGL**: Industry-standard language with mature graphics API
- **🎮 C# + MonoGame**: Productive language with game-focused framework

!!! tip "Learning Strategy"
    Follow all tracks conceptually to understand different approaches, but implement in your chosen language. The architecture decisions are language-agnostic.

## Prerequisites

**Mathematics:**
- Vectors and matrices (See [Mathematical Foundations](math/))
- 3D transformations (translation, rotation, scale)
- Basic linear algebra

**Programming:**
- Intermediate proficiency in your chosen language
- Understanding of structs/classes and memory management
- Basic knowledge of graphics concepts (vertices, triangles, shaders)

**Tools:**
- **Rust Track**: Rust 1.70+, Cargo
- **C++ Track**: C++17 compiler (GCC/Clang/MSVC), CMake
- **C# Track**: .NET 6+, Visual Studio or Rider

---

## Part 1: Foundation - Window and Context

### Architecture Decision: Platform Abstraction

**Goal**: Create a window and initialize graphics context without directly using platform-specific APIs.

**Design Rationale:**

- **Cross-platform windowing** is complex (Win32, X11, Cocoa APIs differ vastly)
- **Use battle-tested libraries** rather than reimplementing platform layer
- **Focus learning on engine architecture**, not OS internals

**Library Choices:**

| Track | Windowing | Graphics | Why |
|-------|-----------|----------|-----|
| Rust | `winit` | `wgpu` | Modern, safe, WebGPU-based abstraction |
| C++ | `GLFW` | `OpenGL` | Industry standard, extensive documentation |
| C# | `MonoGame` | `MonoGame` | Integrated framework, XNA successor |

### Step 1.1: Project Setup

=== "Rust + wgpu"

    **Create Project:**
    
    ```bash
    cargo new my_engine --bin
    cd my_engine
    ```
    
    **Cargo.toml:**
    
    ```toml
    [package]
    name = "my_engine"
    version = "0.1.0"
    edition = "2021"
    
    [dependencies]
    winit = "0.29"
    wgpu = "0.18"
    pollster = "0.3"  # For blocking on async
    env_logger = "0.11"
    log = "0.4"
    glam = "0.24"  # Math library
    bytemuck = { version = "1.14", features = ["derive"] }
    ```
    
    **Why these dependencies?**
    - `winit`: Cross-platform window creation (supports Windows, Linux, macOS, Web)
    - `wgpu`: WebGPU implementation, abstracts over Vulkan/Metal/DX12/OpenGL
    - `glam`: Fast, SIMD-optimized math library
    - `bytemuck`: Safe casting between types for GPU data

=== "C++ + OpenGL"

    **Create Project Structure:**
    
    ```bash
    mkdir my_engine
    cd my_engine
    mkdir src include lib
    ```
    
    **CMakeLists.txt:**
    
    ```cmake
    cmake_minimum_required(VERSION 3.15)
    project(MyEngine)
    
    set(CMAKE_CXX_STANDARD 17)
    set(CMAKE_CXX_STANDARD_REQUIRED ON)
    
    # Find OpenGL
    find_package(OpenGL REQUIRED)
    
    # GLFW (window/input)
    add_subdirectory(lib/glfw)
    
    # GLAD (OpenGL loader)
    add_library(glad lib/glad/src/glad.c)
    target_include_directories(glad PUBLIC lib/glad/include)
    
    # GLM (math)
    add_subdirectory(lib/glm)
    
    # Engine executable
    add_executable(my_engine
        src/main.cpp
        src/engine.cpp
        src/window.cpp
    )
    
    target_include_directories(my_engine PRIVATE include)
    target_link_libraries(my_engine
        OpenGL::GL
        glfw
        glad
        glm
    )
    ```
    
    **Setup Libraries:**
    
    Download and place in `lib/`:
    - [GLFW](https://www.glfw.org/) - Window/input
    - [GLAD](https://glad.dav1d.de/) - OpenGL loader (generate with GL 4.5+ core)
    - [GLM](https://github.com/g-truc/glm) - Math library

=== "C# + MonoGame"

    **Create Project:**
    
    ```bash
    dotnet new install MonoGame.Templates.CSharp
    dotnet new mgdesktopgl -o MyEngine
    cd MyEngine
    ```
    
    **MyEngine.csproj:**
    
    ```xml
    <Project Sdk="Microsoft.NET.Sdk">
      <PropertyGroup>
        <OutputType>WinExe</OutputType>
        <TargetFramework>net6.0</TargetFramework>
        <RollForward>Major</RollForward>
        <PublishReadyToRun>false</PublishReadyToRun>
        <TieredCompilation>false</TieredCompilation>
      </PropertyGroup>
      
      <ItemGroup>
        <PackageReference Include="MonoGame.Framework.DesktopGL" Version="3.8.1.*" />
        <PackageReference Include="MonoGame.Content.Builder.Task" Version="3.8.1.*" />
      </ItemGroup>
    </Project>
    ```
    
    **Why MonoGame?**
    - Built on OpenGL (cross-platform)
    - Content pipeline for asset processing
    - Similar to XNA (familiar to many developers)

### Step 1.2: Create Window

**Architecture Pattern: Game Loop**

All game engines follow the same basic structure:

```
Initialize
Loop:
  - Process Events (input, window resize, etc.)
  - Update (game logic, physics, AI)
  - Render (draw to screen)
Until exit
Cleanup
```

=== "Rust + wgpu"

    **src/main.rs:**
    
    ```rust
    use winit::{
        event::*,
        event_loop::{EventLoop, ControlFlow},
        window::{WindowBuilder, Window},
    };
    
    struct Engine {
        window: Window,
    }
    
    impl Engine {
        fn new(window: Window) -> Self {
            Self { window }
        }
        
        fn handle_event(&mut self, event: &WindowEvent) -> bool {
            match event {
                WindowEvent::CloseRequested => return false,
                WindowEvent::Resized(physical_size) => {
                    log::info!("Window resized: {:?}", physical_size);
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        if keycode == VirtualKeyCode::Escape {
                            return false;
                        }
                    }
                }
                _ => {}
            }
            true
        }
        
        fn update(&mut self, dt: f32) {
            // Game logic will go here
        }
        
        fn render(&mut self) {
            // Rendering will go here
        }
    }
    
    fn main() {
        env_logger::init();
        
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("My Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap();
        
        let mut engine = Engine::new(window);
        let mut last_frame = std::time::Instant::now();
        
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            
            match event {
                Event::WindowEvent { event, .. } => {
                    if !engine.handle_event(&event) {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::MainEventsCleared => {
                    // Update and render
                    let now = std::time::Instant::now();
                    let dt = (now - last_frame).as_secs_f32();
                    last_frame = now;
                    
                    engine.update(dt);
                    engine.render();
                }
                _ => {}
            }
        });
    }
    ```
    
    **Architecture Notes:**
    - `EventLoop`: Receives OS events (input, resize, close)
    - `ControlFlow::Poll`: Run continuously (for games)
    - `ControlFlow::Wait`: Sleep until event (for tools)
    - Delta time (`dt`): Frame-to-frame time for smooth motion

=== "C++ + OpenGL"

    **include/window.h:**
    
    ```cpp
    #pragma once
    #include <GLFW/glfw3.h>
    #include <string>
    
    class Window {
    public:
        Window(int width, int height, const std::string& title);
        ~Window();
        
        bool shouldClose() const;
        void pollEvents();
        void swapBuffers();
        
        int getWidth() const { return width_; }
        int getHeight() const { return height_; }
        GLFWwindow* getHandle() const { return window_; }
        
    private:
        GLFWwindow* window_;
        int width_, height_; 
        
        static void framebufferResizeCallback(GLFWwindow* window, int width, int height);
    };
    ```
    
    **src/window.cpp:**
    
    ```cpp
    #include "window.h"
    #include <glad/glad.h>
    #include <iostream>
    #include <stdexcept>
    
    Window::Window(int width, int height, const std::string& title)
        : width_(width), height_(height) {
        
        if (!glfwInit()) {
            throw std::runtime_error("Failed to initialize GLFW");
        }
        
        // Request OpenGL 4.5 Core Profile
        glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
        glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 5);
        glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
        #ifdef __APPLE__
        glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
        #endif
        
        window_ = glfwCreateWindow(width, height, title.c_str(), nullptr, nullptr);
        if (!window_) {
            glfwTerminate();
            throw std::runtime_error("Failed to create GLFW window");
        }
        
        glfwMakeContextCurrent(window_);
        glfwSetWindowUserPointer(window_, this);
        glfwSetFramebufferSizeCallback(window_, framebufferResizeCallback);
        
        // Load OpenGL functions via GLAD
        if (!gladLoadGLLoader((GLADloadproc)glfwGetProcAddress)) {
            throw std::runtime_error("Failed to initialize GLAD");
        }
        
        std::cout << "OpenGL Version: " << glGetString(GL_VERSION) << std::endl;
    }
    
    Window::~Window() {
        glfwDestroyWindow(window_);
        glfwTerminate();
    }
    
    bool Window::shouldClose() const {
        return glfwWindowShouldClose(window_);
    }
    
    void Window::pollEvents() {
        glfwPollEvents();
    }
    
    void Window::swapBuffers() {
        glfwSwapBuffers(window_);
    }
    
    void Window::framebufferResizeCallback(GLFWwindow* window, int width, int height) {
        auto* self = static_cast<Window*>(glfwGetWindowUserPointer(window));
        self->width_ = width;
        self->height_ = height;
        glViewport(0, 0, width, height);
    }
    ```
    
    **src/main.cpp:**
    
    ```cpp
    #include "window.h"
    #include <glad/glad.h>
    #include <iostream>
    #include <chrono>
    
    class Engine {
    public:
        Engine(int width, int height)
            : window_(width, height, "My Engine") {}
        
        void run() {
            auto lastFrame = std::chrono::high_resolution_clock::now();
            
            while (!window_.shouldClose()) {
                auto now = std::chrono::high_resolution_clock::now();
                float dt = std::chrono::duration<float>(now - lastFrame).count();
                lastFrame = now;
                
                window_.pollEvents();
                
                update(dt);
                render();
                
                window_.swapBuffers();
            }
        }
        
    private:
        Window window_;
        
        void update(float dt) {
            // Game logic will go here
        }
        
        void render() {
            // Clear screen
            glClearColor(0.1f, 0.1f, 0.1f, 1.0f);
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            
            // Rendering will go here
        }
    };
    
    int main() {
        try {
            Engine engine(1280, 720);
            engine.run();
        } catch (const std::exception& e) {
            std::cerr << "Error: " << e.what() << std::endl;
            return 1;
        }
        return 0;
    }
    ```
    
    **Architecture Notes:**
    - GLFW callbacks: Window events handled via function pointers
    - Context management: `glfwMakeContextCurrent()` binds OpenGL to window
    - Double buffering: Draw to back buffer, swap to front

=== "C# + MonoGame"

    **Game1.cs:**
    
    ```csharp
    using Microsoft.Xna.Framework;
    using Microsoft.Xna.Framework.Graphics;
    using Microsoft.Xna.Framework.Input;
    using System;
    
    namespace MyEngine
    {
        public class Game1 : Game
        {
            private GraphicsDeviceManager _graphics;
            private SpriteBatch _spriteBatch;
            
            public Game1()
            {
                _graphics = new GraphicsDeviceManager(this);
                Content.RootDirectory = "Content";
                IsMouseVisible = true;
                
                // Set window size
                _graphics.PreferredBackBufferWidth = 1280;
                _graphics.PreferredBackBufferHeight = 720;
            }
            
            protected override void Initialize()
            {
                // Initialize game systems here
                Console.WriteLine("Engine Initialized");
                base.Initialize();
            }
            
            protected override void LoadContent()
            {
                _spriteBatch = new SpriteBatch(GraphicsDevice);
                // Load assets here
            }
            
            protected override void Update(GameTime gameTime)
            {
                if (GamePad.GetState(PlayerIndex.One).Buttons.Back == ButtonState.Pressed ||
                    Keyboard.GetState().IsKeyDown(Keys.Escape))
                {
                    Exit();
                }
                
                // Game logic here
                float dt = (float)gameTime.ElapsedGameTime.TotalSeconds;
                
                base.Update(gameTime);
            }
            
            protected override void Draw(GameTime gameTime)
            {
                GraphicsDevice.Clear(Color.CornflowerBlue);
                
                // Rendering will go here
                
                base.Draw(gameTime);
            }
        }
    }
    ```
    
    **Program.cs:**
    
    ```csharp
    using System;
    
    namespace MyEngine
    {
        public static class Program
        {
            [STAThread]
            static void Main()
            {
                using var game = new Game1();
                game.Run();
            }
        }
    }
    ```
    
    **Architecture Notes:**
    - MonoGame provides complete game loop
    - `Initialize()`: One-time setup
    - `Update()`: Fixed timestep by default (60Hz)
    - `Draw()`: Variable timestep rendering
    - `GameTime`: Provides total and elapsed time

**Key Takeaway**: At this point, you have a window that clears to a solid color. This is your canvas for everything that follows.

---

## Part 2: Game Loop Architecture

### Architecture Decision: Fixed vs Variable Timestep

**The Problem**: Different computers run at different speeds. How do we ensure consistent simulation?

**Approaches:**

1. **Variable Timestep**: Use actual elapsed time
   - ✅ Smooth rendering
   - ❌ Physics instability, non-deterministic

2. **Fixed Timestep**: Update in constant increments
   - ✅ Stable physics, deterministic, networkable
   - ❌ Requires interpolation for smooth rendering

3. **Semi-Fixed (Recommended)**: Fixed update, variable render
   - ✅ Best of both worlds
   - ❌ Slightly more complex

**Implementation Pattern:**

```
accumulator = 0
FIXED_DT = 1/60  // 60 Hz physics

loop:
  frame_time = current_time - last_time
  accumulator += frame_time
  
  while accumulator >= FIXED_DT:
    update_physics(FIXED_DT)
    accumulator -= FIXED_DT
  
  alpha = accumulator / FIXED_DT
  render(interpolate(previous_state, current_state, alpha))
```

**Why This Matters:**

- **Physics engines** (like Praxis uses Rapier) require fixed timestep for stability
- **Networked games** need determinism for client prediction
- **Gameplay** benefits from consistent timing (jump height, projectile speed)

**For This Tutorial**: We'll use variable timestep initially for simplicity, but understand that production engines typically use semi-fixed timestep.

---

## Part 3: Input System

### Architecture Decision: Input State Management

**Design Goals:**
- Query input state from any system
- Detect button press/release events (not just "held")
- Support keyboard and mouse initially
- Extensible to gamepad later

**Pattern: Input State Snapshot**

```
Frame N-1: Space=false
Frame N:   Space=true   → "Just Pressed"
Frame N+1: Space=true   → "Held"
Frame N+2: Space=false  → "Just Released"
```

### Step 3.1: Implement Input System

=== "Rust + wgpu"

    **Create src/input.rs:**
    
    ```rust
    use winit::event::{VirtualKeyCode, ElementState, MouseButton};
    use std::collections::HashSet;
    use glam::Vec2;
    
    pub struct InputState {
        keys_pressed: HashSet<VirtualKeyCode>,
        keys_just_pressed: HashSet<VirtualKeyCode>,
        keys_just_released: HashSet<VirtualKeyCode>,
        
        mouse_buttons_pressed: HashSet<MouseButton>,
        mouse_buttons_just_pressed: HashSet<MouseButton>,
        mouse_buttons_just_released: HashSet<MouseButton>,
        
        mouse_position: Vec2,
        mouse_delta: Vec2,
    }
    
    impl InputState {
        pub fn new() -> Self {
            Self {
                keys_pressed: HashSet::new(),
                keys_just_pressed: HashSet::new(),
                keys_just_released: HashSet::new(),
                mouse_buttons_pressed: HashSet::new(),
                mouse_buttons_just_pressed: HashSet::new(),
                mouse_buttons_just_released: HashSet::new(),
                mouse_position: Vec2::ZERO,
                mouse_delta: Vec2::ZERO,
            }
        }
        
        pub fn update(&mut self) {
            // Clear per-frame state
            self.keys_just_pressed.clear();
            self.keys_just_released.clear();
            self.mouse_buttons_just_pressed.clear();
            self.mouse_buttons_just_released.clear();
            self.mouse_delta = Vec2::ZERO;
        }
        
        pub fn handle_keyboard(&mut self, keycode: VirtualKeyCode, state: ElementState) {
            match state {
                ElementState::Pressed => {
                    if self.keys_pressed.insert(keycode) {
                        self.keys_just_pressed.insert(keycode);
                    }
                }
                ElementState::Released => {
                    self.keys_pressed.remove(&keycode);
                    self.keys_just_released.insert(keycode);
                }
            }
        }
        
        pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
            match state {
                ElementState::Pressed => {
                    if self.mouse_buttons_pressed.insert(button) {
                        self.mouse_buttons_just_pressed.insert(button);
                    }
                }
                ElementState::Released => {
                    self.mouse_buttons_pressed.remove(&button);
                    self.mouse_buttons_just_released.insert(button);
                }
            }
        }
        
        pub fn handle_mouse_move(&mut self, position: Vec2) {
            self.mouse_delta = position - self.mouse_position;
            self.mouse_position = position;
        }
        
        // Query methods
        pub fn is_key_pressed(&self, keycode: VirtualKeyCode) -> bool {
            self.keys_pressed.contains(&keycode)
        }
        
        pub fn is_key_just_pressed(&self, keycode: VirtualKeyCode) -> bool {
            self.keys_just_pressed.contains(&keycode)
        }
        
        pub fn is_key_just_released(&self, keycode: VirtualKeyCode) -> bool {
            self.keys_just_released.contains(&keycode)
        }
        
        pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
            self.mouse_buttons_pressed.contains(&button)
        }
        
        pub fn mouse_position(&self) -> Vec2 {
            self.mouse_position
        }
        
        pub fn mouse_delta(&self) -> Vec2 {
            self.mouse_delta
        }
    }
    ```
    
    **Update src/main.rs to integrate input:**
    
    ```rust
    mod input;
    use input::InputState;
    
    struct Engine {
        window: Window,
        input: InputState,
    }
    
    impl Engine {
        fn new(window: Window) -> Self {
            let input = InputState::new();
            Self { window, input }
        }
        
        fn handle_event(&mut self, event: &WindowEvent) -> bool {
            match event {
                WindowEvent::CloseRequested => return false,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        self.input.handle_keyboard(keycode, input.state);
                        if keycode == VirtualKeyCode::Escape {
                            return false;
                        }
                    }
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    self.input.handle_mouse_button(*button, *state);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.input.handle_mouse_move(glam::vec2(
                        position.x as f32,
                        position.y as f32,
                    ));
                }
                _ => {}
            }
            true
        }
        
        fn begin_frame(&mut self) {
            self.input.update();
        }
        
        fn update(&mut self, dt: f32) {
            if self.input.is_key_just_pressed(VirtualKeyCode::Space) {
                log::info!("Space pressed!");
            }
        }
    }
    ```

=== "C++ + OpenGL"

    **include/input.h:**
    
    ```cpp
    #pragma once
    #include <GLFW/glfw3.h>
    #include <glm/glm.hpp>
    #include <unordered_set>
    
    class InputState {
    public:
        void update();
        
        void setKeyState(int key, bool pressed);
        void setMouseButtonState(int button, bool pressed);
        void setMousePosition(float x, float y);
        
        bool isKeyPressed(int key) const;
        bool isKeyJustPressed(int key) const;
        bool isKeyJustReleased(int key) const;
        
        bool isMouseButtonPressed(int button) const;
        glm::vec2 getMousePosition() const { return mousePosition_; }
        glm::vec2 getMouseDelta() const { return mouseDelta_; }
        
    private:
        std::unordered_set<int> keysPressed_;
        std::unordered_set<int> keysJustPressed_;
        std::unordered_set<int> keysJustReleased_;
        
        std::unordered_set<int> mouseButtonsPressed_;
        std::unordered_set<int> mouseButtonsJustPressed_;
        std::unordered_set<int> mouseButtonsJustReleased_;
        
        glm::vec2 mousePosition_{0.0f, 0.0f};
        glm::vec2 mouseDelta_{0.0f, 0.0f};
    };
    ```
    
    **src/input.cpp:**
    
    ```cpp
    #include "input.h"
    
    void InputState::update() {
        keysJustPressed_.clear();
        keysJustReleased_.clear();
        mouseButtonsJustPressed_.clear();
        mouseButtonsJustReleased_.clear();
        mouseDelta_ = glm::vec2(0.0f);
    }
    
    void InputState::setKeyState(int key, bool pressed) {
        if (pressed) {
            if (keysPressed_.insert(key).second) {
                keysJustPressed_.insert(key);
            }
        } else {
            keysPressed_.erase(key);
            keysJustReleased_.insert(key);
        }
    }
    
    void InputState::setMouseButtonState(int button, bool pressed) {
        if (pressed) {
            if (mouseButtonsPressed_.insert(button).second) {
                mouseButtonsJustPressed_.insert(button);
            }
        } else {
            mouseButtonsPressed_.erase(button);
            mouseButtonsJustReleased_.insert(button);
        }
    }
    
    void InputState::setMousePosition(float x, float y) {
        glm::vec2 newPos(x, y);
        mouseDelta_ = newPos - mousePosition_;
        mousePosition_ = newPos;
    }
    
    bool InputState::isKeyPressed(int key) const {
        return keysPressed_.count(key) > 0;
    }
    
    bool InputState::isKeyJustPressed(int key) const {
        return keysJustPressed_.count(key) > 0;
    }
    
    bool InputState::isKeyJustReleased(int key) const {
        return keysJustReleased_.count(key) > 0;
    }
    
    bool InputState::isMouseButtonPressed(int button) const {
        return mouseButtonsPressed_.count(button) > 0;
    }
    ```

=== "C# + MonoGame"

    **Create Input/InputState.cs:**
    
    ```csharp
    using Microsoft.Xna.Framework;
    using Microsoft.Xna.Framework.Input;
    
    namespace MyEngine.Input
    {
        public class InputState
        {
            private KeyboardState _currentKeyboardState;
            private KeyboardState _previousKeyboardState;
            
            private MouseState _currentMouseState;
            private MouseState _previousMouseState;
            
            public void Update()
            {
                _previousKeyboardState = _currentKeyboardState;
                _currentKeyboardState = Keyboard.GetState();
                
                _previousMouseState = _currentMouseState;
                _currentMouseState = Mouse.GetState();
            }
            
            public bool IsKeyPressed(Keys key)
            {
                return _currentKeyboardState.IsKeyDown(key);
            }
            
            public bool IsKeyJustPressed(Keys key)
            {
                return _currentKeyboardState.IsKeyDown(key) && 
                       _previousKeyboardState.IsKeyUp(key);
            }
            
            public bool IsKeyJustReleased(Keys key)
            {
                return _currentKeyboardState.IsKeyUp(key) && 
                       _previousKeyboardState.IsKeyDown(key);
            }
            
            public bool IsMouseButtonPressed(MouseButton button)
            {
                return button switch
                {
                    MouseButton.Left => _currentMouseState.LeftButton == ButtonState.Pressed,
                    MouseButton.Right => _currentMouseState.RightButton == ButtonState.Pressed,
                    MouseButton.Middle => _currentMouseState.MiddleButton == ButtonState.Pressed,
                    _ => false
                };
            }
            
            public Vector2 MousePosition => new Vector2(_currentMouseState.X, _currentMouseState.Y);
            
            public Vector2 MouseDelta => new Vector2(
                _currentMouseState.X - _previousMouseState.X,
                _currentMouseState.Y - _previousMouseState.Y
            );
        }
        
        public enum MouseButton
        {
            Left,
            Right,
            Middle
        }
    }
    ```

---

## Conclusion and Next Steps

This guide has walked you through the fundamental architecture decisions and initial implementation of a game engine from scratch. You've learned:

1. **Platform Abstraction**: Why and how to use windowing libraries
2. **Game Loop Structure**: The initialize-update-render pattern
3. **Input Management**: Frame-based input state tracking
4. **Architecture Choices**: The rationale behind different approaches

### What You've Built

At this point, you have:
- A cross-platform window
- A running game loop with delta time
- Input handling system for keyboard and mouse
- Foundation for graphics initialization

### Recommended Next Steps

To continue building your engine, you should implement:

1. **Graphics Rendering** (Part 4 - Coming Soon)
   - GPU initialization and command buffers
   - Shader pipeline setup
   - Vertex buffers and rendering your first triangle
   - Camera system and 3D transformations

2. **Resource Management** (Part 5 - Coming Soon)
   - Mesh loading (OBJ format)
   - Texture loading and sampling
   - Material system
   - Asset lifetime management

3. **3D Scene** (Part 6 - Coming Soon)
   - Transform hierarchies
   - Scene graph structure
   - Multiple objects rendering
   - Basic lighting

4. **Physics Integration** (Part 7 - Coming Soon)
   - Integrating a physics library
   - Collision detection
   - Rigidbody simulation
   - Fixed timestep implementation

### Learning Resources

**For Deep Dives:**
- [Praxis Curriculum](CURRICULUM.md) - Comprehensive engine architecture course
- [Mathematical Foundations](math/) - 3D math essentials
- [Universal Patterns](patterns/) - Design patterns across engines

**External References:**
- **Game Engine Architecture** by Jason Gregory - Industry bible
- **Real-Time Rendering** by Akenine-Möller et al. - Graphics fundamentals
- **Learn OpenGL** (learnopengl.com) - Excellent graphics tutorials
- **Vulkan Tutorial** (vulkan-tutorial.com) - Modern graphics API

### Philosophy: Learning by Building

Building an engine from scratch teaches you:
- **How engines work internally**: No more "magic" abstractions
- **Graphics API concepts**: Fundamental to all 3D programming
- **Systems thinking**: How subsystems interact and depend on each other
- **Performance awareness**: Where bottlenecks occur and why
- **Design trade-offs**: Why engines make certain architectural choices

**When to Use Your Engine vs. Existing Engines:**

- **Use your engine for**: Learning, experimentation, specific requirements, full control
- **Use existing engines for**: Shipping games, team collaboration, built-in tools, proven tech

### Contributing

This guide is part of the Praxis educational project. If you:
- Find errors or unclear explanations
- Want to add implementation examples for other languages (Python, JavaScript, Go, etc.)
- Have suggestions for additional sections
- Build something cool with this guide

Please contribute! The goal is to make engine architecture accessible to everyone.

---

**Ready to continue?** Watch for Parts 4-7 covering graphics rendering, resource management, 3D scenes, and physics. Each part will follow the same multi-language approach with detailed architecture explanations.

**Questions or feedback?** Open an issue on the Praxis repository or contribute improvements to this guide.
