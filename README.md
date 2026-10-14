# χrust-md

[![crates.io](https://img.shields.io/crates/v/xrust-md.svg)](https://crates.io/crates/xrust-md)
[![Released API docs](https://docs.rs/xrust-md/badge.svg)](https://docs.rs/xrust-md)
[![ALv2 licensed](https://img.shields.io/badge/license-ALv2-blue.svg)](./LICENSE)

A parser for Markdown documents that creates a [χrust](https://gitlab.gnome.org/World/Rust/markup-rs/xrust/) tree.

## Markdown Editing

Markdown is simply text with some embedded instructions.

### Headings

A level 1 heading is denoted by a '#' at the beginning of the line.
```
# A Level 1 Heading
```
A level 2 heading is denoted by a '##' at the beginning of the line.
```
## A Level 2 Heading
```

There is no limit to heading levels.

### Paragraphs

The end of the line starts a new paragraph.
```
The first paragraph.
Followed by another paragraph.
```

### Phrases

For strong (bold) text surround the text with '**'.
For underlined text surround the text with '__' (underscores).
For emphasised (italic) text surround the texy with '//'.
```
The **strong** phrase followed by an __underlined__ phrase and then //italics// and then the end of the paragraph.
```

## Usage

This crate is used by [χrusty](https://gitlab.gnome.org/World/Rust/markup-rs/xrusty/) to support Markdown as a source document in a transformation.

## Change Log

| Release | Notes |
|---------|-------|
| 0.2.3   | Synchronise with xrust 2.3.0 |
|---------|-------|
| 0.2.0   | Unlimited heading levels |
|---------|-------|
| 0.1.0   | Initial release |
|---------|-------|
