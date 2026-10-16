use ripex::{parser_for_ext, CommentKind, Language};

#[test]
fn test_rust_comments_and_docstrings() {
    let source = r#"// Module line comment
/* Block comment */

/// Represents a worker process.
pub struct Worker {
    pub id: u32,
}

impl Worker {
    /// Creates a new worker instance.
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}
// Trailing EOF comment"#;

    let parser = parser_for_ext("rust", "rs").unwrap();
    let result = parser.parse(source);
    let facts = parser.extract(&result).unwrap();

    assert_eq!(result.language, Language::Rust);
    assert_eq!(result.status, ripex::ParseStatus::Complete);
    assert!(result.comments.len() >= 4);

    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Line && c.text.contains("Module line comment")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Block && c.text.contains("Block comment")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Doc && c.text.contains("Represents a worker")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Line && c.text.contains("Trailing EOF comment")));

    let worker = facts
        .symbols
        .iter()
        .find(|s| s.name == "Worker")
        .expect("Worker struct");
    assert_eq!(
        worker.doc_string.as_deref(),
        Some("Represents a worker process.")
    );

    let new_fn = facts
        .symbols
        .iter()
        .find(|s| s.name == "new")
        .expect("new function");
    assert_eq!(
        new_fn.doc_string.as_deref(),
        Some("Creates a new worker instance.")
    );
}

#[test]
fn test_python_comments_and_docstrings() {
    let source = r#"# Module-level comment
import os

class Geometry:
    """Class computing geometry figures."""

    def distance(self, x: float, y: float) -> float:
        """Compute euclidean distance between points."""
        return (x * x + y * y) ** 0.5
# Trailing comment"#;

    let parser = parser_for_ext("python", "py").unwrap();
    let result = parser.parse(source);
    let facts = parser.extract(&result).unwrap();

    assert_eq!(result.language, Language::Python);
    assert_eq!(result.status, ripex::ParseStatus::Complete);
    assert!(result.comments.len() >= 2);

    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Line && c.text.contains("Module-level comment")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Line && c.text.contains("Trailing comment")));

    let geom = facts
        .symbols
        .iter()
        .find(|s| s.name == "Geometry")
        .expect("Geometry class");
    assert_eq!(
        geom.doc_string.as_deref(),
        Some("Class computing geometry figures.")
    );

    let dist = facts
        .symbols
        .iter()
        .find(|s| s.name == "distance")
        .expect("distance method");
    assert_eq!(
        dist.doc_string.as_deref(),
        Some("Compute euclidean distance between points.")
    );
}

#[test]
fn test_go_comments_and_docstrings() {
    let source = r#"// Package server implements http handlers.
package server

// Config holds runtime server options.
type Config struct {
    Port int
}

// Start boots the server listener.
func Start(c *Config) {
}
// End of file comment"#;

    let parser = parser_for_ext("go", "go").unwrap();
    let result = parser.parse(source);
    let facts = parser.extract(&result).unwrap();

    assert_eq!(result.language, Language::Go);
    assert_eq!(result.status, ripex::ParseStatus::Complete);
    assert!(result.comments.len() >= 3);

    assert!(result
        .comments
        .iter()
        .any(|c| c.text.contains("Package server")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.text.contains("End of file")));

    let config = facts
        .symbols
        .iter()
        .find(|s| s.name == "Config")
        .expect("Config type");
    assert_eq!(
        config.doc_string.as_deref(),
        Some("Config holds runtime server options.")
    );

    let start = facts
        .symbols
        .iter()
        .find(|s| s.name == "Start")
        .expect("Start func");
    assert_eq!(
        start.doc_string.as_deref(),
        Some("Start boots the server listener.")
    );
}

#[test]
fn test_c_comments_and_docstrings() {
    let source = r#"/* Global C header comment */
#include <stdio.h>

/** Record represents an entity with an ID. */
typedef struct {
    int id;
} Record;

/** Process the record. */
int process(Record* rec) {
    return rec->id;
}
// Trailing line comment"#;

    let parser = parser_for_ext("c", "c").unwrap();
    let result = parser.parse(source);
    let facts = parser.extract(&result).unwrap();

    assert_eq!(result.language, Language::C);
    assert_eq!(result.status, ripex::ParseStatus::Complete);
    assert!(result.comments.len() >= 4);

    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Block && c.text.contains("Global C header")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Doc && c.text.contains("Record represents")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Doc && c.text.contains("Process the record")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Line && c.text.contains("Trailing line")));

    let record = facts
        .symbols
        .iter()
        .find(|s| s.name == "Record")
        .expect("Record struct");
    assert_eq!(
        record.doc_string.as_deref(),
        Some("Record represents an entity with an ID.")
    );

    let proc = facts
        .symbols
        .iter()
        .find(|s| s.name == "process")
        .expect("process func");
    assert_eq!(proc.doc_string.as_deref(), Some("Process the record."));
}

#[test]
fn test_cpp_comments_and_docstrings() {
    let source = r#"// C++ engine header
#include <iostream>

namespace engine {
/** Engine task execution unit. */
class Task {
public:
    /** Execute the background task. */
    void run() {}
};
}
// EOF comment"#;

    let parser = parser_for_ext("cpp", "cpp").unwrap();
    let result = parser.parse(source);
    let facts = parser.extract(&result).unwrap();

    assert_eq!(result.language, Language::Cpp);
    assert_eq!(result.status, ripex::ParseStatus::Complete);
    assert!(result.comments.len() >= 3);

    assert!(result
        .comments
        .iter()
        .any(|c| c.text.contains("C++ engine header")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Doc && c.text.contains("Engine task")));

    let task = facts
        .symbols
        .iter()
        .find(|s| s.name == "Task")
        .expect("Task class");
    assert_eq!(
        task.doc_string.as_deref(),
        Some("Engine task execution unit.")
    );
}

#[test]
fn test_csharp_comments_and_docstrings() {
    let source = r#"// C# architecture file
using System;

namespace Enterprise
{
    /// <summary>
    /// Worker engine component.
    /// </summary>
    public class Worker
    {
        /// <summary>
        /// Perform work unit.
        /// </summary>
        public void DoWork() {}
    }
}
// Final C# comment"#;

    let parser = parser_for_ext("csharp", "cs").unwrap();
    let result = parser.parse(source);
    let facts = parser.extract(&result).unwrap();

    assert_eq!(result.language, Language::CSharp);
    assert_eq!(result.status, ripex::ParseStatus::Complete);
    assert!(result.comments.len() >= 3);

    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Doc && c.text.contains("Worker engine")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.text.contains("Final C# comment")));

    let worker = facts
        .symbols
        .iter()
        .find(|s| s.name.contains("Worker"))
        .expect("Worker class");
    assert!(worker
        .doc_string
        .as_deref()
        .unwrap()
        .contains("Worker engine component."));
}

#[test]
fn test_javascript_and_typescript_comments_and_docstrings() {
    let source = r#"#! /usr/bin/env node
// Leading JS comment
/* Multi-line block comment */

/** Calculator class with operations */
export class Calculator {
    /** Calculates the addition of two numbers */
    add(a, b) {
        return a + b;
    }
}
// End of file comment"#;

    let parser = parser_for_ext("javascript", "js").unwrap();
    let result = parser.parse(source);
    let facts = parser.extract(&result).unwrap();

    assert_eq!(result.language, Language::JavaScript);
    assert_eq!(result.status, ripex::ParseStatus::Complete);
    assert!(result.comments.len() >= 4);

    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Hashbang));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Line && c.text.contains("Leading JS")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Block && c.text.contains("Multi-line block")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Doc && c.text.contains("Calculator class")));
    assert!(result
        .comments
        .iter()
        .any(|c| c.kind == CommentKind::Line && c.text.contains("End of file")));

    let calc = facts
        .symbols
        .iter()
        .find(|s| s.name == "Calculator")
        .expect("Calculator symbol");
    assert_eq!(
        calc.doc_string.as_deref(),
        Some("Calculator class with operations")
    );
}
