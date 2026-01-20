# Exercise 41: OBJ File Parser

**Difficulty**: 🟡 Intermediate | **Estimated Time**: 4-5h | **Subsystem**: Assets

## Overview

Implement a parser for the Wavefront OBJ 3D model format. OBJ is a simple, text-based format widely used for 3D assets, making it perfect for learning asset pipeline basics.

## Learning Objectives

- Understand 3D model file formats
- Implement efficient text parsing
- Handle mesh topology (vertices, faces, normals, UVs)
- Learn error handling for malformed input

## Requirements

### Functional Requirements

1. **Supported Features**
   - Vertex positions (`v`)
   - Texture coordinates (`vt`)
   - Vertex normals (`vn`)
   - Faces (`f`) - triangles and quads
   - Object groups (`o` and `g`)
   - Comments (`#`)

2. **Face Formats**
   - `f v1 v2 v3` (position only)
   - `f v1/vt1 v2/vt2 v3/vt3` (position/texcoord)
   - `f v1/vt1/vn1 v2/vt2/vn2 v3/vt3/vn3` (position/texcoord/normal)
   - `f v1//vn1 v2//vn2 v3//vn3` (position/normal, no texcoord)

3. **Output Format**
   - Indexed mesh (vertex buffer + index buffer)
   - Optionally triangulate quads
   - Generate normals if missing

### Non-Functional Requirements

- **Performance**: Parse 10MB OBJ file in < 500ms
- **Memory**: Streaming parse (don't load entire file into memory)
- **Robustness**: Handle malformed input gracefully

## API Design

```rust
pub struct ObjParser;

impl ObjParser {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Mesh, ParseError>;
    pub fn parse_str(content: &str) -> Result<Mesh, ParseError>;
}

pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

pub enum ParseError {
    IoError(std::io::Error),
    InvalidFormat { line: usize, message: String },
    MissingData { line: usize, field: String },
}
```

## Validation Criteria

### Correctness
- [ ] Correctly parses all vertex types (v, vt, vn)
- [ ] Handles all face formats
- [ ] Converts OBJ indices (1-based) to 0-based
- [ ] Triangulates quads correctly
- [ ] Reports line numbers in errors

### Performance
- [ ] Parse 10MB file in < 500ms
- [ ] Memory usage proportional to mesh size
- [ ] No unnecessary allocations

## Test Cases

```rust
#[test]
fn test_simple_triangle() {
    let obj = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
"#;
    
    let mesh = ObjParser::parse_str(obj).unwrap();
    assert_eq!(mesh.positions.len(), 3);
    assert_eq!(mesh.indices.len(), 3);
    assert_eq!(mesh.indices, vec![0, 1, 2]);
}

#[test]
fn test_quad_triangulation() {
    let obj = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
f 1 2 3 4
"#;
    
    let mesh = ObjParser::parse_str(obj).unwrap();
    assert_eq!(mesh.indices.len(), 6); // 2 triangles = 6 indices
    assert_eq!(mesh.indices, vec![0, 1, 2, 0, 2, 3]);
}

#[test]
fn test_with_normals_and_uvs() {
    let obj = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
f 1/1/1 2/2/2 3/3/3
"#;
    
    let mesh = ObjParser::parse_str(obj).unwrap();
    assert_eq!(mesh.positions.len(), 3);
    assert_eq!(mesh.normals.len(), 3);
    assert_eq!(mesh.texcoords.len(), 3);
}

#[test]
fn test_negative_indices() {
    let obj = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f -3 -2 -1
"#;
    
    let mesh = ObjParser::parse_str(obj).unwrap();
    assert_eq!(mesh.indices, vec![0, 1, 2]);
}

#[test]
fn test_error_reporting() {
    let obj = "v 0.0 invalid 0.0";
    let result = ObjParser::parse_str(obj);
    assert!(result.is_err());
}
```

## Performance Targets

| File Size | Target Parse Time |
|-----------|-------------------|
| 1MB | < 50ms |
| 10MB | < 500ms |
| 100MB | < 5s |

## Hints & Guidance

### OBJ Format Quick Reference
```
# Comment
v x y z [w]              # Vertex position
vt u v [w]               # Texture coordinate
vn x y z                 # Vertex normal
f v1/vt1/vn1 v2/vt2/vn2 ... # Face

# Indices are 1-based
# Negative indices count from end (-1 = last vertex)
```

### Parsing Strategy
1. Read line by line
2. Split on whitespace
3. Match first token to determine line type
4. Parse remaining tokens as floats or face indices

### Face Index Parsing
```rust
// "1/2/3" -> (1, Some(2), Some(3))
// "1//3"  -> (1, None, Some(3))
// "1"     -> (1, None, None)
```

### Triangulation
- Quad `a b c d` becomes two triangles: `a b c` and `a c d`
- This is fan triangulation from first vertex

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct ObjParser;

impl ObjParser {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Mesh, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::parse_reader(reader)
    }
    
    pub fn parse_str(content: &str) -> Result<Mesh, ParseError> {
        let cursor = std::io::Cursor::new(content);
        Self::parse_reader(BufReader::new(cursor))
    }
    
    fn parse_reader<R: BufRead>(reader: R) -> Result<Mesh, ParseError> {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut texcoords = Vec::new();
        
        let mut out_positions = Vec::new();
        let mut out_normals = Vec::new();
        let mut out_texcoords = Vec::new();
        let mut indices = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }
            
            match tokens[0] {
                "v" => {
                    // Vertex position
                    if tokens.len() < 4 {
                        return Err(ParseError::InvalidFormat {
                            line: line_num + 1,
                            message: "Vertex requires 3 coordinates".to_string(),
                        });
                    }
                    let x = tokens[1].parse::<f32>().map_err(|_| ParseError::InvalidFormat {
                        line: line_num + 1,
                        message: "Invalid float".to_string(),
                    })?;
                    let y = tokens[2].parse::<f32>().map_err(|_| ParseError::InvalidFormat {
                        line: line_num + 1,
                        message: "Invalid float".to_string(),
                    })?;
                    let z = tokens[3].parse::<f32>().map_err(|_| ParseError::InvalidFormat {
                        line: line_num + 1,
                        message: "Invalid float".to_string(),
                    })?;
                    positions.push([x, y, z]);
                }
                
                "vt" => {
                    // Texture coordinate
                    if tokens.len() < 3 {
                        return Err(ParseError::InvalidFormat {
                            line: line_num + 1,
                            message: "Texture coordinate requires 2 values".to_string(),
                        });
                    }
                    let u = tokens[1].parse::<f32>().map_err(|_| ParseError::InvalidFormat {
                        line: line_num + 1,
                        message: "Invalid float".to_string(),
                    })?;
                    let v = tokens[2].parse::<f32>().map_err(|_| ParseError::InvalidFormat {
                        line: line_num + 1,
                        message: "Invalid float".to_string(),
                    })?;
                    texcoords.push([u, v]);
                }
                
                "vn" => {
                    // Vertex normal
                    if tokens.len() < 4 {
                        return Err(ParseError::InvalidFormat {
                            line: line_num + 1,
                            message: "Normal requires 3 coordinates".to_string(),
                        });
                    }
                    let x = tokens[1].parse::<f32>().map_err(|_| ParseError::InvalidFormat {
                        line: line_num + 1,
                        message: "Invalid float".to_string(),
                    })?;
                    let y = tokens[2].parse::<f32>().map_err(|_| ParseError::InvalidFormat {
                        line: line_num + 1,
                        message: "Invalid float".to_string(),
                    })?;
                    let z = tokens[3].parse::<f32>().map_err(|_| ParseError::InvalidFormat {
                        line: line_num + 1,
                        message: "Invalid float".to_string(),
                    })?;
                    normals.push([x, y, z]);
                }
                
                "f" => {
                    // Face
                    if tokens.len() < 4 {
                        return Err(ParseError::InvalidFormat {
                            line: line_num + 1,
                            message: "Face requires at least 3 vertices".to_string(),
                        });
                    }
                    
                    let mut face_indices = Vec::new();
                    for i in 1..tokens.len() {
                        let (v_idx, vt_idx, vn_idx) = parse_face_vertex(tokens[i], line_num + 1)?;
                        
                        // Convert to 0-based indices
                        let v_idx = convert_index(v_idx, positions.len())?;
                        
                        // Build unique vertex
                        out_positions.push(positions[v_idx]);
                        
                        if let Some(vt) = vt_idx {
                            let vt_idx = convert_index(vt, texcoords.len())?;
                            out_texcoords.push(texcoords[vt_idx]);
                        }
                        
                        if let Some(vn) = vn_idx {
                            let vn_idx = convert_index(vn, normals.len())?;
                            out_normals.push(normals[vn_idx]);
                        }
                        
                        face_indices.push((out_positions.len() - 1) as u32);
                    }
                    
                    // Triangulate if needed (fan triangulation)
                    for i in 1..(face_indices.len() - 1) {
                        indices.push(face_indices[0]);
                        indices.push(face_indices[i]);
                        indices.push(face_indices[i + 1]);
                    }
                }
                
                _ => {
                    // Ignore unknown lines
                }
            }
        }
        
        Ok(Mesh {
            positions: out_positions,
            normals: out_normals,
            texcoords: out_texcoords,
            indices,
        })
    }
}

fn parse_face_vertex(s: &str, line: usize) -> Result<(i32, Option<i32>, Option<i32>), ParseError> {
    let parts: Vec<&str> = s.split('/').collect();
    
    let v_idx = parts[0].parse::<i32>().map_err(|_| ParseError::InvalidFormat {
        line,
        message: format!("Invalid vertex index: {}", parts[0]),
    })?;
    
    let vt_idx = if parts.len() > 1 && !parts[1].is_empty() {
        Some(parts[1].parse::<i32>().map_err(|_| ParseError::InvalidFormat {
            line,
            message: format!("Invalid texcoord index: {}", parts[1]),
        })?)
    } else {
        None
    };
    
    let vn_idx = if parts.len() > 2 && !parts[2].is_empty() {
        Some(parts[2].parse::<i32>().map_err(|_| ParseError::InvalidFormat {
            line,
            message: format!("Invalid normal index: {}", parts[2]),
        })?)
    } else {
        None
    };
    
    Ok((v_idx, vt_idx, vn_idx))
}

fn convert_index(idx: i32, len: usize) -> Result<usize, ParseError> {
    if idx > 0 {
        Ok((idx - 1) as usize)
    } else if idx < 0 {
        let pos = (len as i32 + idx) as usize;
        Ok(pos)
    } else {
        Err(ParseError::InvalidFormat {
            line: 0,
            message: "Index cannot be 0".to_string(),
        })
    }
}

pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    InvalidFormat { line: usize, message: String },
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(e) => write!(f, "IO error: {}", e),
            ParseError::InvalidFormat { line, message } => {
                write!(f, "Parse error at line {}: {}", line, message)
            }
        }
    }
}

impl std::error::Error for ParseError {}
```

</details>

## Related Resources

- [OBJ Format Specification](http://paulbourke.net/dataformats/obj/)
- [Praxis Assets Documentation](../../reference/crates.md#praxis_assets)
- [Asset Loading Benchmark](../../benchmarking.md#asset-loading)

## Next Steps

- Add GLTF parser (more complex format)
- Implement async loading (Exercise 43)
- Add normal generation for smooth shading
